use crate::UptimeInfo;
use std::fs;
use std::path::PathBuf;

pub fn get_uptime() -> UptimeInfo {
    UptimeInfo {
        uptime: 0,
        time_system_started: 0,
    }
}

pub fn get_recent_file_macos() -> String {
    "maybe will implement later".to_string()
}

pub fn get_installed_apps() -> Vec<String> {
    
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