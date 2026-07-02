use crate::UptimeInfo;
use winreg::enums::*;
use winreg::RegKey;

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
        time_system_started: current_time - total_seconds,
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

        let Ok(metadata) = entry.metadata() 
        else 
        { 
            continue; 
        };
        
        let Ok(last_modified) = metadata.modified() 
        else 
        { 
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
    
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let paths = [
        (hklm.clone(), "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (hklm, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (hkcu, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    for location in paths {
        let root_key = location.0;
        let folder_path = location.1;
        
        let opened_folder = root_key.open_subkey(folder_path);
        
        if let Ok(key) = opened_folder {
            for name_result in key.enum_keys() {
                if let Ok(name) = name_result {
                    let opened_subkey = key.open_subkey(&name);
                    
                    if let Ok(subkey) = opened_subkey {
                        let display_name_result: Result<String, _> = subkey.get_value("DisplayName");
                        
                        if let Ok(display_name) = display_name_result {
                            let clean_name = display_name.trim().to_string();
                            
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
    }
    
    // The sorting logic is now built directly into this function using a readable, multi-line block.
    apps.sort_by(|a, b| {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        a_lower.cmp(&b_lower)
    });
    
    return apps;
}