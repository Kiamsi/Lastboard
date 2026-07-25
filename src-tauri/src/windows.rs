use crate::UptimeInfo;
use winreg::enums::*;
use winreg::RegKey;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetTcp6Table2, GetTcpTable2, MIB_TCP6TABLE2, MIB_TCPTABLE2,
};
use std::process::Command;
use windows_sys::Win32::System::Performance::{
    PdhAddCounterW, PdhCollectQueryData, PdhGetFormattedCounterValue, PdhOpenQueryW,
    PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE, PDH_FMT_LARGE, PDH_HCOUNTER, PDH_HQUERY,
};

pub fn get_uptime() -> UptimeInfo {
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;

    let milliseconds = unsafe { GetTickCount64() };
    let total_seconds = milliseconds / 1000;
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("i know you didn't turn on your pc in the 60s")
        .as_secs();

    UptimeInfo {
        uptime: total_seconds,
        time_system_started: current_time.saturating_sub(total_seconds),
    }
}

const TCP_ESTABLISHED: u32 = 5;

pub fn get_open_connections() -> usize {
    let mut total = 0;

    unsafe {
        // ipv4
        let mut size: u32 = 0;
        GetTcpTable2(std::ptr::null_mut(), &mut size, 0);
        if size > 0 {
           
            size += 256;
            let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
            let table_ptr = buffer.as_mut_ptr() as *mut MIB_TCPTABLE2;

            if GetTcpTable2(table_ptr, &mut size, 0) == 0 {
                let table = &*table_ptr;
                let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
                total += rows.iter().filter(|row| row.dwState == TCP_ESTABLISHED).count();
            }
        }

        // ipv6
        let mut size: u32 = 0;
        GetTcp6Table2(std::ptr::null_mut(), &mut size, 0);
        if size > 0 {
            size += 256;
            let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
            let table_ptr = buffer.as_mut_ptr() as *mut MIB_TCP6TABLE2;

            if GetTcp6Table2(table_ptr, &mut size, 0) == 0 {
                let table = &*table_ptr;
                let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
                total += rows.iter().filter(|row| row.State as u32 == TCP_ESTABLISHED).count();
            }
        }
    }

    total
}

const TCP_LISTEN: u32 = 2;

pub fn get_listening_ports() -> usize {
    let mut total = 0;

    unsafe {
        let mut size: u32 = 0;
        GetTcpTable2(std::ptr::null_mut(), &mut size, 0);
        if size > 0 {
            size += 256;
            let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
            let table_ptr = buffer.as_mut_ptr() as *mut MIB_TCPTABLE2;

            if GetTcpTable2(table_ptr, &mut size, 0) == 0 {
                let table = &*table_ptr;
                let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
                total += rows.iter().filter(|row| row.dwState == TCP_LISTEN).count();
            }
        }

        let mut size: u32 = 0;
        GetTcp6Table2(std::ptr::null_mut(), &mut size, 0);
        if size > 0 {
            size += 256;
            let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
            let table_ptr = buffer.as_mut_ptr() as *mut MIB_TCP6TABLE2;

            if GetTcp6Table2(table_ptr, &mut size, 0) == 0 {
                let table = &*table_ptr;
                let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
                total += rows.iter().filter(|row| row.State as u32 == TCP_LISTEN).count();
            }
        }
    }

    total
}

pub fn get_connected_lan_devices() -> usize {
    let output = match Command::new("arp").arg("-a").output() {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout.lines()
        .filter(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() == 3
                && parts[1].contains('-')
                && parts[1] != "00-00-00-00-00-00"
                && parts[1] != "ff-ff-ff-ff-ff-ff"
        })
        .count()
}

pub struct DiskIoQuery {
    query: PDH_HQUERY,
    read_counter: PDH_HCOUNTER,
    write_counter: PDH_HCOUNTER,
}

unsafe impl Send for DiskIoQuery {}
unsafe impl Sync for DiskIoQuery {}

pub fn init_disk_io_query() -> DiskIoQuery {
    unsafe {
        let mut query: PDH_HQUERY = std::ptr::null_mut();
        PdhOpenQueryW(std::ptr::null(), 0, &mut query);

        let read_path: Vec<u16> = "\\PhysicalDisk(_Total)\\Disk Read Bytes/sec"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let write_path: Vec<u16> = "\\PhysicalDisk(_Total)\\Disk Write Bytes/sec"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut read_counter: PDH_HCOUNTER = std::ptr::null_mut();
        let mut write_counter: PDH_HCOUNTER = std::ptr::null_mut();
        PdhAddCounterW(query, read_path.as_ptr(), 0, &mut read_counter);
        PdhAddCounterW(query, write_path.as_ptr(), 0, &mut write_counter);


        PdhCollectQueryData(query);

        DiskIoQuery { query, read_counter, write_counter }
    }
}

pub fn poll_disk_io(state: &mut DiskIoQuery) -> (u64, u64) {
    unsafe {
        PdhCollectQueryData(state.query);

        let mut read_value: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
        let mut write_value: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
        PdhGetFormattedCounterValue(state.read_counter, PDH_FMT_LARGE, std::ptr::null_mut(), &mut read_value);
        PdhGetFormattedCounterValue(state.write_counter, PDH_FMT_LARGE, std::ptr::null_mut(), &mut write_value);

        let read_bytes = if read_value.CStatus == PDH_CSTATUS_VALID_DATA {
            read_value.Anonymous.largeValue.max(0) as u64
        } else {
            0
        };
        let write_bytes = if write_value.CStatus == PDH_CSTATUS_VALID_DATA {
            write_value.Anonymous.largeValue.max(0) as u64
        } else {
            0
        };

        (read_bytes, write_bytes)
    }
}

pub fn installed_apps() -> Vec<String> {
    let mut apps = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let paths = [
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
    ];

    for (root_key, folder_path) in paths {
        let Ok(key) = root_key.open_subkey(folder_path) else {
            continue;
        };

        for name in key.enum_keys().flatten() {
            let Ok(subkey) = key.open_subkey(&name) else {
                continue;
            };

            let display_name_result: Result<String, _> = subkey.get_value("DisplayName");
            let Ok(display_name) = display_name_result else {
                continue;
            };

            let clean_name = display_name.trim().to_string();
            if clean_name.is_empty() {
                continue;
            }

            if seen.insert(clean_name.to_lowercase()) {
                apps.push(clean_name);
            }
        }
    }

    apps.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    apps
}

pub fn get_recent_file_windows() -> String {
    let appdata = std::env::var("APPDATA").expect("no appdata folder, something's very wrong");
    let recent_folder = format!("{}\\Microsoft\\Windows\\Recent", appdata);

    let entries = match std::fs::read_dir(&recent_folder) {
        Ok(entries) => entries,
        Err(_) => return String::from("can't read recent folder"),
    };

    let mut best_name = String::from("nothing found");
    let mut best_time = std::time::SystemTime::UNIX_EPOCH;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if !file_name.ends_with(".lnk") {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(last_modified) = metadata.modified() else {
            continue;
        };

        if last_modified > best_time {
            best_time = last_modified;
            best_name = file_name.trim_end_matches(".lnk").to_string();
        }
    }

    best_name
}

pub fn get_os_name() -> String {
    let cmd_result = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').ProductName")
        .output();

    if cmd_result.is_err() {
        return String::from("Windows");
    }

    let output = cmd_result.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let name = stdout.trim();

    if name.is_empty() {
        return String::from("Windows");
    }

    String::from(name)
}

pub fn get_os_version() -> String {
    let cmd_result = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').DisplayVersion")
        .output();

    if cmd_result.is_err() {
        return String::from("Unknown");
    }

    let output = cmd_result.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.trim();

    if version.is_empty() {
        return String::from("Unknown");
    }

    String::from(version)
}

pub fn get_connected_peripherals() -> usize {
    let cmd_result = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("(Get-PnpDevice -PresentOnly | Where-Object { $_.InstanceId -like 'USB\\*' -and $_.Class -ne 'USB' } | Measure-Object).Count")
        .output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<usize>().unwrap_or(0)
}

pub fn get_last_system_update() -> u64 {
    let cmd_result = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("$s = New-Object -ComObject Microsoft.Update.Session; $r = $s.CreateUpdateSearcher(); $h = $r.QueryHistory(0, $r.GetTotalHistoryCount()); $last = $h | Where-Object { $_.ResultCode -eq 2 -and $_.Date } | Sort-Object Date -Descending | Select-Object -First 1; if ($last) { ([DateTimeOffset]$last.Date).ToUnixTimeSeconds() }")
        .output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<u64>().unwrap_or(0)
}

pub fn get_monitors() -> usize {
    let cmd_result = Command::new("powershell").arg("-NoProfile").arg("-Command")
        .arg("Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Screen]::AllScreens.Count")
        .output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<usize>().unwrap_or(0)
}