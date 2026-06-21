#[derive(serde::Serialize)]
struct UptimeInfo {
    uptime_formatted: String,
    boot_unix_seconds: u64,
}



#[cfg(target_os = "windows")]
fn get_recent_file_windows() -> String {
    //gets the latest edited file on windows

    //takes the APPDATA env variable
    let appdata = std::env::var("APPDATA").expect("no appdata folder, something's very wrong");
    let recent_folder = format!("{}\\Microsoft\\Windows\\Recent", appdata);

    //iterates over the recent folder
    let entries = match std::fs::read_dir(&recent_folder) {
        Ok(entries) => entries,
        Err(_) => return String::from("can't read recent folder"),
    };

    let mut best_name = String::from("nothing found");
    let mut best_time = std::time::SystemTime::UNIX_EPOCH; //the date 1970.01.01

    //finds the most recently opened file that matches
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        let last_modified = match metadata.modified() {
            Ok(time) => time,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.ends_with(".lnk") && last_modified > best_time {
            best_time = last_modified;
            best_name = file_name.trim_end_matches(".lnk").to_string();
        }
    }
    best_name
}

#[tauri::command]
fn get_recent_file_os() -> String {
    #[cfg(target_os = "windows")]
    {
        get_recent_file_windows()
    }

    #[cfg(target_os = "linux")]
    {
        "might implement later".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "idk if i'll implement it later".to_string()
    }
}

//necessary to port to mobile even if it won't be used
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_recent_file_os])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
