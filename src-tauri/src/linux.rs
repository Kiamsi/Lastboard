use crate::UptimeInfo;
use crate::ConnectionInfo;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

pub fn get_uptime() -> UptimeInfo {
    
    let uptime_contents = match fs::read_to_string("/proc/uptime") {
        Ok(text) => text,
        Err(_) => String::from("0.0"), //fallback if the file can't be read
    };

    // the file has two numbers separated by a space and we only want the first one
    let mut words = uptime_contents.split_whitespace();
    
    let first_word = match words.next() {
        Some(word) => word,
        None => "0.0", //fallback if the file was empty
    };

    let total_seconds = match first_word.parse::<f64>() {
        Ok(decimal_number) => decimal_number.round() as u64,
        Err(_) => 0, 
    };

    let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH);
    
    let current_time = match time_since_epoch {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0, 
    };

    let start_time = current_time.saturating_sub(total_seconds);

    UptimeInfo {
        uptime: total_seconds,
        time_system_started: start_time,
    }
}

fn count_connected(contents: &str) -> usize {
    contents.lines().skip(1) .filter(|line| {
            
            line.split_whitespace().nth(3) == Some("01")
        })
        .count()
}
pub fn get_open_connections() -> usize {
    let tcp4 = fs::read_to_string("/proc/net/tcp")
        .expect("/proc/net/tcp should always exist");
    let mut total = count_connected(&tcp4);

    
    if let Ok(tcp6) = fs::read_to_string("/proc/net/tcp6") {
        total += count_connected(&tcp6);
    }

    total
}

pub fn get_listening_ports() -> usize {
    let tcp4 = std::fs::read_to_string("/proc/net/tcp").unwrap_or_default();
    let tcp6 = std::fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
    
    let count_listening = |contents: &str| -> usize {
        contents.lines() .skip(1).filter(|line| {
                
                line.split_whitespace().nth(3) == Some("0A")
            })
            .count()
    };
    
    count_listening(&tcp4) + count_listening(&tcp6)
}

pub fn get_connected_lan_devices() -> usize {
    let contents = match fs::read_to_string("/proc/net/arp") {
        Ok(text) => text, Err(_) => return 0,
    };
    contents.lines().skip(1)
    .filter(|line| line.split_whitespace().nth(3) != Some("00:00:00:00:00:00")).count()
}

pub fn installed_apps() -> Vec<String> {
    
    let mut apps = Vec::new();
    let home_directory = std::env::var("HOME").unwrap_or_default();
    let mut paths_to_check = Vec::new();

    let data_home = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", home_directory));
    paths_to_check.push(PathBuf::from(data_home).join("applications"));

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    
    for dir in data_dirs.split(':') {
        
        if !dir.is_empty() {
            paths_to_check.push(PathBuf::from(dir).join("applications"));
        }
    }

    paths_to_check.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    paths_to_check.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    for current_path in paths_to_check {
        
        let directory_contents = fs::read_dir(current_path);
        
        //oh man
        if let Ok(entries) = directory_contents {
            
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    
                    let file_path = entry.path();
                    let mut is_desktop_file = false;
                    
                    if let Some(extension) = file_path.extension() {
                        
                        if extension == "desktop" {
                            is_desktop_file = true;
                        }
                    }
                    
                    if is_desktop_file {
                        let file_content_result = fs::read_to_string(&file_path);
                       
                        if let Ok(content) = file_content_result {
                            let mut name = None;
                            let mut skip_entry = false;

                            for line in content.lines() {

                                if line.starts_with('[') && line != "[Desktop Entry]" {
                                    break;
                                }
                                if let Some(rest) = line.strip_prefix("Name=") {
                                    name = Some(rest.trim().to_string());
                                } else if line == "NoDisplay=true" || line == "Hidden=true" {
                                    skip_entry = true;
                                }
                            }

                            if !skip_entry {
                                if let Some(clean_name) = name {
                                    if !clean_name.is_empty()
                                        && !apps.iter().any(|a: &String| a.eq_ignore_ascii_case(&clean_name))
                                    {
                                        apps.push(clean_name);
                                    }
                                }
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

pub fn get_recent_file_linux() -> String {
    "might make later".to_string()
}

pub fn get_os_name() -> String {
    let file_result = std::fs::read_to_string("/etc/os-release");
    
    if file_result.is_err() {
        return String::from("Linux");
    }
    
    let contents = file_result.unwrap();
    
    for line in contents.lines() {
        if line.starts_with("PRETTY_NAME=") {
            let mut name = line.replace("PRETTY_NAME=", "");
            name = name.replace("\"", "");
            name = name.replace("'", "");
            return name;
        }
    }
    
    for line in contents.lines() {
        if line.starts_with("NAME=") {
            let mut name = line.replace("NAME=", "");
            name = name.replace("\"", "");
            name = name.replace("'", "");
            return name;
        }
    }
    
    String::from("Linux")
}

pub fn get_os_version() -> String {
    let file_result = std::fs::read_to_string("/etc/os-release");

    if file_result.is_err() {
        return String::from("Unknown");
    }

    let contents = file_result.unwrap();

    for line in contents.lines() {
        if line.starts_with("VERSION_ID=") {
            let mut version = line.replace("VERSION_ID=", "");
            version = version.replace("\"", "");
            version = version.replace("'", "");
            return version;
        }
    }

    //rolling releases like arch don't set version id but do set build id so im going with it
    for line in contents.lines() {
        if line.starts_with("BUILD_ID=") {
            let build = line.replace("BUILD_ID=", "").replace("\"", "").replace("'", "");
            if build.eq_ignore_ascii_case("rolling") {
                return String::from("Rolling Release");
            }
            return build;
        }
    }

    String::from("Unknown")
}

fn real_block_devices() -> Vec<String> {
    let mut names = Vec::new();
    let entries = match fs::read_dir("/sys/block") {
        Ok(e) => e,
        Err(_) => return names,
    };
    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
    
        if name.starts_with("loop") || name.starts_with("dm-")|| name.starts_with("ram") || name.starts_with("zram")
        {
            continue;
        }
        names.push(name);
    }
    names
}

pub fn read_disk_totals() -> (u64, u64) {
    const SECTOR_SIZE: u64 = 512;

    let contents = match fs::read_to_string("/proc/diskstats") {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let devices = real_block_devices();
    let mut total_read_sectors = 0u64;
    let mut total_written_sectors = 0u64;

    for line in contents.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }
        let name = fields[2];
        if !devices.iter().any(|d| d == name) {
            continue; 
        }
        total_read_sectors += fields[5].parse::<u64>().unwrap_or(0);
        total_written_sectors += fields[9].parse::<u64>().unwrap_or(0);
    }

    (total_read_sectors * SECTOR_SIZE, total_written_sectors * SECTOR_SIZE)
}

pub fn get_connected_peripherals() -> usize {
    
    let entries = match fs::read_dir("/sys/bus/usb/devices") {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut count = 0;

    for entry_result in entries {
        
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();

        if name.contains(':') {
            continue;
        }

        if name.starts_with("usb") {
            continue;
        }

        let class = fs::read_to_string(entry.path().join("bDeviceClass")).unwrap_or_default();
        if class.trim() == "09" {
            continue;
        }

        count += 1;
    }

    count
}

pub fn get_last_system_update() -> u64 {
    let paths = [
        "/var/lib/pacman/local",
        "/var/log/apt/history.log",
        "/var/lib/rpm",
        "/var/log/zypper.log",
        "/var/log/dpkg.log",
    ];

    for path in paths {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    return duration.as_secs();
                }
            }
        }
    }

    0
}

pub fn get_monitors() -> usize {
    
    let entries = match fs::read_dir("/sys/class/drm") {
        Ok(e) => e,Err(_) => return 0,
    };

    let mut count = 0;

    for entry_result in entries {
        
        let entry = match entry_result {
            Ok(e) => e, Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();

        
        if !name.contains('-') {
            continue;
        }

        let status = fs::read_to_string(entry.path().join("status")).unwrap_or_default();
        if status.trim() == "connected" {
            count += 1;
        }
    }

    count
}

use std::collections::HashMap;

pub fn get_last_disk_writer(previous_totals: &mut HashMap<i32, u64>) -> String {
   
    let proc_directory = match std::fs::read_dir("/proc") {
        Ok(directory) => directory, Err(_) => return String::from("can't read /proc"),
    };

    let mut busiest_pid: i32 = 0;
    
    let mut busiest_delta: u64 = 0;
    
    let mut new_totals: HashMap<i32, u64> = HashMap::new();

    for entry in proc_directory {
        
        let entry = match entry {
            Ok(e) => e, Err(_) => continue,
        };

        let file_name = entry.file_name();
        
        let name_str = file_name.to_string_lossy();

        let pid: i32 = match name_str.parse() {
            Ok(p) => p, Err(_) => continue, 
        };

        let io_path = format!("/proc/{}/io", pid);
       
        let contents = match std::fs::read_to_string(&io_path) {
            Ok(text) => text, Err(_) => continue, 
        };

        let mut write_bytes: u64 = 0;
        
        for line in contents.lines() {
            
            if line.starts_with("write_bytes:") {
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                
                if parts.len() == 2 {
                    
                    if let Ok(value) = parts[1].parse::<u64>() {
                        write_bytes = value;
                    }
                }
                break;
            }
        }

        new_totals.insert(pid, write_bytes);

        let previous_value = match previous_totals.get(&pid) {
            Some(value) => *value, None => write_bytes,
        };

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
        return String::from("(Writing processes)");
    }

    let comm_path = format!("/proc/{}/comm", busiest_pid);
    
    let process_name = match std::fs::read_to_string(&comm_path) {
        Ok(name) => name.trim().to_string(),  Err(_) => String::from("unknown"),
    };

    format!("{} (pid {})", process_name, busiest_pid)
}

pub fn get_last_sleep_time() -> String {
    let output = match Command::new("journalctl")
        .args(["-u", "systemd-suspend.service", "-o", "json", "-n", "1", "--no-pager"])
        .output()
    {
        Ok(result) => result,
        Err(_) => return String::from("could not run journalctl"),
    };

    let log_text = String::from_utf8_lossy(&output.stdout);
    let search_key = "\"__REALTIME_TIMESTAMP\"";
    let mut timestamp_seconds: u64 = 0;

    for line in log_text.lines() {
        let Some(key_pos) = line.find(search_key) else {
            continue;
        };
        let after_key = &line[key_pos + search_key.len()..];

        let Some(colon_pos) = after_key.find(':') else {
            continue;
        };
        let after_colon = &after_key[colon_pos + 1..];

        let Some(quote_start) = after_colon.find('"') else {
            continue;
        };
        let value_slice = &after_colon[quote_start + 1..];

        let Some(quote_end) = value_slice.find('"') else {
            continue;
        };
        let timestamp_text = &value_slice[..quote_end];

        let Ok(timestamp_micros) = timestamp_text.parse::<u64>() else {
            continue;
        };

        timestamp_seconds = timestamp_micros / 1_000_000;
    }

    if timestamp_seconds == 0 {
        return String::from("no sleep event found in the log");
    }

    let formatted = match Command::new("date")
        .arg("-d")
        .arg(format!("@{timestamp_seconds}"))
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
    {
        Ok(result) => result,
        Err(_) => return String::from("could not format sleep time"),
    };

    String::from_utf8_lossy(&formatted.stdout).trim().to_string()
}

pub fn get_connections() -> Vec<ConnectionInfo> {
    let mut connections = Vec::new();
    connections.extend(read_proc_net("/proc/net/tcp", "TCP", false));
    connections.extend(read_proc_net("/proc/net/tcp6", "TCP", true));
    connections.extend(read_proc_net("/proc/net/udp", "UDP", false));
    connections.extend(read_proc_net("/proc/net/udp6", "UDP", true));
    connections
}

fn read_proc_net(path: &str, protocol: &str, is_v6: bool) -> Vec<ConnectionInfo> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut connections = Vec::new();

    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }

        let Some((local_hex, local_port_hex)) = fields[1].split_once(':') else {
            continue;
        };
        let Some((remote_hex, remote_port_hex)) = fields[2].split_once(':') else {
            continue;
        };

        let (local_address, remote_address) = if is_v6 {
            (parse_hex_addr_v6(local_hex), parse_hex_addr_v6(remote_hex))
        } else {
            (parse_hex_addr_v4(local_hex), parse_hex_addr_v4(remote_hex))
        };

        let state = if protocol == "TCP" {
            tcp_state_name(fields[3])
        } else {
            String::new()
        };

        connections.push(ConnectionInfo {
            protocol: protocol.to_string(),
            local_address,
            local_port: parse_hex_port(local_port_hex),
            remote_address,
            remote_port: parse_hex_port(remote_port_hex),
            state,
        });
    }

    connections
}

fn parse_hex_addr_v4(hex_str: &str) -> String {
    let val = u32::from_str_radix(hex_str, 16).unwrap_or(0);
    let bytes = val.to_le_bytes();
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

fn parse_hex_addr_v6(hex_str: &str) -> String {
    if hex_str.len() != 32 {
        return hex_str.to_string();
    }
    let mut bytes = [0u8; 16];
    for i in 0..4 {
        let chunk = &hex_str[i * 8..i * 8 + 8];
        let val = u32::from_str_radix(chunk, 16).unwrap_or(0);
        bytes[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
    }
    std::net::Ipv6Addr::from(bytes).to_string()
}

fn parse_hex_port(hex_str: &str) -> u16 {
    u16::from_str_radix(hex_str, 16).unwrap_or(0)
}

fn tcp_state_name(code: &str) -> String {
    match code {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    }
    .to_string()
}

pub fn get_last_installed_app() -> String {
    
    last_installed_apt().or_else(last_installed_pacman).or_else(last_installed_rpm)
    .unwrap_or_default()
}

fn last_installed_apt() -> Option<String> {
    for path in ["/var/log/dpkg.log", "/var/log/dpkg.log.1"] {
        if let Some(app) = parse_dpkg_log(path) {
            return Some(app);
        }
    }
    None
}

fn parse_dpkg_log(path: &str) -> Option<String> {
   
    let contents = std::fs::read_to_string(path).ok()?;

    for line in contents.lines().rev() {
       
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() >= 4 && fields[2] == "install" {
           
            let pkg_name = fields[3].split(':').next().unwrap_or(fields[3]);
            
            return Some(pkg_name.to_string());
        }
    }

    None
}

fn last_installed_pacman() -> Option<String> {
   
    let contents = std::fs::read_to_string("/var/log/pacman.log").ok()?;

    for line in contents.lines().rev() {
        
        let Some(rest) = line.split("[ALPM] installed ").nth(1) else {
            continue;
        };
        
        let Some(pkg_name) = rest.split_whitespace().next() else {
            continue;
        };
        return Some(pkg_name.to_string());
    }

    None
}

fn last_installed_rpm() -> Option<String> {
    
    let output = Command::new("rpm").arg("-qa").arg("--queryformat").arg("%{INSTALLTIME} %{NAME}\n")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut newest_time: u64 = 0;
   
    let mut newest_name = String::new();

    for line in stdout.lines() {
        
        let mut parts = line.splitn(2, ' ');
        
        let Some(time_str) = parts.next() else { continue };
        
        let Some(name) = parts.next() else { continue };
        
        let Ok(time) = time_str.parse::<u64>() else { continue };

        if time >= newest_time {
            newest_time = time;
            newest_name = name.to_string();
        }
    }

    if newest_name.is_empty() {
        None
    } else {
        Some(newest_name)
    }
}