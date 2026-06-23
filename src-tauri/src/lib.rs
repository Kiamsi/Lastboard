#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[derive(serde::Serialize)]
pub struct UptimeInfo {
    pub uptime: u64,
    pub time_system_started: u64,
}

#[tauri::command]
fn get_uptime() -> UptimeInfo {
    #[cfg(target_os = "windows")]
    {
        windows::get_uptime()
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::get_uptime()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_uptime()
    }
}

#[tauri::command]
fn get_process_count() -> usize {
    use sysinfo::System;
    let mut system = System::new_all();
    system.refresh_all();
    system.processes().len()
}

#[tauri::command]
fn get_recent_file_os() -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_recent_file_windows()
    }

    #[cfg(target_os = "linux")]
    {
        linux::get_recent_file_linux()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_recent_file_macos()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_recent_file_os, 
            get_uptime, 
            get_process_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}