use crate::UptimeInfo;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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