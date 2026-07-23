use crate::UptimeInfo;
use winreg::enums::*;
use winreg::RegKey;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetTcp6Table2, GetTcpTable2, MIB_TCP6TABLE2, MIB_TCPTABLE2,
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

const TCP_ESTABLISHED: u32 = 5;

pub fn get_open_connections() -> usize {
    
    let mut total = 0;

    unsafe {
        // ipv4
        let mut size: u32 = 0;
        GetTcpTable2(std::ptr::null_mut(), &mut size, 0);
        if size > 0 {
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