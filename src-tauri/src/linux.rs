use crate::UptimeInfo;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

pub fn get_uptime() -> UptimeInfo {
    // 1. Read the raw text from the Linux uptime file
    let uptime_contents = match fs::read_to_string("/proc/uptime") {
        Ok(text) => text,
        Err(_) => String::from("0.0"), // Fallback if the file can't be read
    };

    // 2. The file has two numbers separated by a space. We only want the first one.
    let mut words = uptime_contents.split_whitespace();
    let first_word = match words.next() {
        Some(word) => word,
        None => "0.0", // Fallback if the file was empty
    };

    // 3. Convert the text (e.g., "350735.47") into a decimal number, then round to a whole number
    let total_seconds = match first_word.parse::<f64>() {
        Ok(decimal_number) => decimal_number as u64,
        Err(_) => 0, // Fallback if the text wasn't a valid number
    };

    // 4. Figure out exactly what time it is right now
    let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH);
    let current_time = match time_since_epoch {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0, // Fallback in the rare event the system clock goes backwards
    };

    // 5. Calculate when the system started and return the data
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
    paths_to_check.push(PathBuf::from("/usr/share/applications"));
    paths_to_check.push(PathBuf::from(format!("{}/.local/share/applications", home_directory)));
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
                            for line in content.lines() {
                                if line.starts_with("Name=") {
                                    let clean_name = line.replace("Name=", "").trim().to_string();
                                    
                                    if !clean_name.is_empty() {
                                        if !apps.contains(&clean_name) {
                                            apps.push(clean_name);
                                        }
                                    }
                                    
                                    break;
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