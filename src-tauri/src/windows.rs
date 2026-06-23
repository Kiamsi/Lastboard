use crate::UptimeInfo;

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