use crate::UptimeInfo;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::ffi::c_void;

pub fn get_uptime() -> UptimeInfo {
    let cmd_result = Command::new("sysctl").arg("-n").arg("kern.boottime").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return UptimeInfo { uptime: 0, time_system_started: 0 },
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    let boot_time = stdout
        .split("sec = ")
        .nth(1)
        .and_then(|rest| rest.split(',').next())
        .and_then(|num| num.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    UptimeInfo {
        uptime: current_time.saturating_sub(boot_time),
        time_system_started: boot_time,
    }
}

pub fn installed_apps() -> Vec<String> {
let mut apps = Vec::new();
let home_directory = std::env::var("HOME").unwrap_or_default();
let mut paths_to_check = Vec::new();
paths_to_check.push(PathBuf::from("/Applications"));
paths_to_check.push(PathBuf::from(format!("{}/Applications", home_directory)));

for current_path in paths_to_check {
let directory_contents = fs::read_dir(current_path);
if let Ok(entries) = directory_contents {
for entry_result in entries {
if let Ok(entry) = entry_result {
let raw_name = entry.file_name();
let string_name = raw_name.to_string_lossy().into_owned();
if string_name.ends_with(".app") {
let clean_name = string_name.replace(".app", "");
if !clean_name.is_empty() {
if !apps.contains(&clean_name) {
apps.push(clean_name);
                            }
                        }
                    }
                }
            }
        }
    }
apps.sort_by(|a, b| {
let a_lower = a.to_lowercase();
let b_lower = b.to_lowercase();
a_lower.cmp(&b_lower)
    });
return apps;
}

pub fn get_os_version() -> String {
    let cmd_result = Command::new("sw_vers").arg("-productVersion").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return String::from("Unknown"),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.trim();

    if version.is_empty() {
        String::from("Unknown")
    } else {
        String::from(version)
    }
}

pub fn get_open_connections() -> usize {
    let cmd_result = Command::new("netstat").arg("-an").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| line.starts_with("tcp") && line.contains("ESTABLISHED"))
        .count()
}

pub fn get_listening_ports() -> usize {
    let cmd_result = Command::new("netstat").arg("-an").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| line.starts_with("tcp") && line.contains("LISTEN"))
        .count()
}

pub fn get_connected_lan_devices() -> usize {
    let cmd_result = Command::new("arp").arg("-a").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // format - ? (192.168.1.1) at 3c:e1:a1:xx:xx:xx on en0 ifscope ethernet
            parts.len() > 3
                && parts[2] == "at"
                && parts[3].contains(':')
                && parts[3] != "ff:ff:ff:ff:ff:ff"
        })
        .count()
}

pub fn get_connected_peripherals() -> usize {
    let cmd_result = Command::new("system_profiler").arg("SPUSBDataType").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.ends_with(':'))
        .filter(|line| {
            ![
                "Product ID:", "Vendor ID:", "Version:", "Serial Number:", "Speed:",
                "Manufacturer:", "Location ID:", "Current Available (mA):",
                "Current Required (mA):", "Extra Operating Current (mA):",
                "Capacity:", "Removable Media:", "Detachable Drive:", "BSD Name:",
                "Partition Map Type:", "Volumes:", "Media:", "USB:",
            ]
            .contains(line)
        })
        .filter(|line| !line.ends_with("Bus:"))
        .filter(|line| !line.to_lowercase().contains("hub"))
        .count()
}

pub fn get_last_system_update() -> u64 {
    let metadata = match fs::metadata("/var/log/install.log") {
        Ok(m) => m,
        Err(_) => return 0,
    };

    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return 0,
    };

    modified.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn get_monitors() -> usize {
    let cmd_result = Command::new("system_profiler").arg("SPDisplaysDataType").output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return 0,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("Resolution:"))
        .count()
}

pub fn read_disk_totals() -> (u64, u64) {
    let cmd_result = Command::new("ioreg")
        .arg("-c")
        .arg("IOBlockStorageDriver")
        .arg("-r")
        .arg("-w0")
        .output();

    let output = match cmd_result {
        Ok(out) => out,
        Err(_) => return (0, 0),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut total_read = 0u64;
    let mut total_written = 0u64;

    for line in stdout.lines() {
        if let Some(pos) = line.find("\"Bytes (Read)\"=") {
            let rest = &line[pos + "\"Bytes (Read)\"=".len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            total_read += digits.parse::<u64>().unwrap_or(0);
        }

        if let Some(pos) = line.find("\"Bytes (Write)\"=") {
            let rest = &line[pos + "\"Bytes (Write)\"=".len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            total_written += digits.parse::<u64>().unwrap_or(0);
        }
    }

    (total_read, total_written)
}

pub fn get_last_sleep_time() -> String {
    let output = match Command::new("pmset").arg("-g").arg("log").output() {
        Ok(result) => result,
        Err(_) => return String::from("could not run pmset"),
    };

    let log_text = String::from_utf8_lossy(&output.stdout);

    let mut last_sleep_date = String::new();
    let mut last_sleep_time = String::new();

    for line in log_text.lines() {
        if line.contains("Maintenance") {
            continue;
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }

        if fields[3] == "Sleep" {
            last_sleep_date = fields[0].to_string();
            last_sleep_time = fields[1].to_string();
        }
    }

    if last_sleep_date.is_empty() {
        return String::from("no sleep event found in the log");
    }

    format!("{} {}", last_sleep_date, last_sleep_time)
}

const PROC_ALL_PIDS: u32 = 1;
const RUSAGE_INFO_V2: i32 = 2;

extern "C" {
    fn proc_listpids(proc_type: u32, typeinfo: u32, buffer: *mut c_void, buffersize: i32) -> i32;
    fn proc_name(pid: i32, buffer: *mut c_void, buffersize: u32) -> i32;
    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut c_void) -> i32;
}

//this is dumb but it has to be
#[repr(C)]
#[derive(Default)]
struct RUsageInfoV2 {
    ri_uuid: [u8; 16],
    ri_user_time: u64,
    ri_system_time: u64,
    ri_pkg_idle_wkups: u64,
    ri_interrupt_wkups: u64,
    ri_pageins: u64,
    ri_wired_size: u64,
    ri_resident_size: u64,
    ri_phys_footprint: u64,
    ri_proc_start_abstime: u64,
    ri_proc_exit_abstime: u64,
    ri_child_user_time: u64,
    ri_child_system_time: u64,
    ri_child_pkg_idle_wkups: u64,
    ri_child_interrupt_wkups: u64,
    ri_child_pageins: u64,
    ri_child_elapsed_abstime: u64,
    ri_diskio_bytesread: u64,
    ri_diskio_byteswritten: u64,
}

fn list_all_pids() -> Vec<i32> {
    unsafe {
        // First call with a null buffer just asks for the required size, in bytes.
        let size = proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0);
        if size <= 0 {
            return Vec::new();
        }

        let capacity = size as usize / std::mem::size_of::<i32>();
        let mut pids = vec![0i32; capacity];

        let ret = proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr() as *mut c_void, size);
        if ret <= 0 {
            return Vec::new();
        }

        let actual_count = (ret as usize / std::mem::size_of::<i32>()).min(pids.len());
        pids.truncate(actual_count);
        pids.retain(|&pid| pid != 0);
        pids
    }
}

fn process_name(pid: i32) -> String {
    
    let mut buf = vec![0u8; 256];
    
    let ret = unsafe { proc_name(pid, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };

    if ret <= 0 {
        return String::from("unknown");
    }

   
    buf.truncate(ret as usize);
    String::from_utf8_lossy(&buf).to_string()
}

fn disk_write_bytes(pid: i32) -> Option<u64> {
    
    let mut usage = RUsageInfoV2::default();
    
    let ptr = &mut usage as *mut RUsageInfoV2 as *mut c_void;
    
    let ret = unsafe { proc_pid_rusage(pid, RUSAGE_INFO_V2, ptr) };

    if ret < 0 {
        None
    } else {
        Some(usage.ri_diskio_byteswritten)
    }
}

pub fn get_last_disk_writer(previous_totals: &mut HashMap<i32, u64>) -> String {
    
    let mut busiest_pid: i32 = 0;
   
    let mut busiest_delta: u64 = 0;
    
    let mut new_totals: HashMap<i32, u64> = HashMap::new();

    for pid in list_all_pids() {
        
        let Some(write_bytes) = disk_write_bytes(pid) else {
            continue;
        };

        new_totals.insert(pid, write_bytes);

        let previous_value = previous_totals.get(&pid).copied().unwrap_or(write_bytes);

        if write_bytes > previous_value {
            
            let delta = write_bytes - previous_value;
            
            if delta > busiest_delta {
                busiest_delta = delta;
                busiest_pid = pid;
            }
        }
    }

    *previous_totals = new_totals;

    if busiest_pid == 0 {
        return String::from("No currently writing process");
    }

    format!("{} (pid {})", process_name(busiest_pid), busiest_pid)
}