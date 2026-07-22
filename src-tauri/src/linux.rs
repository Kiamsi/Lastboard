use crate::UptimeInfo;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn get_recent_file_linux() -> String {
    "might implement later".to_string()
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