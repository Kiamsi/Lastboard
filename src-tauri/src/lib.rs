#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use std::sync::Mutex;
use sysinfo::{Networks, System};

#[derive(serde::Serialize)]
pub struct UptimeInfo {
    pub uptime: u64,
    pub time_system_started: u64,
}

pub struct AppState {
    pub system: Mutex<System>,
    pub networks: Mutex<Networks>,
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
fn get_process_count(state: tauri::State<AppState>) -> usize {
    
    let mut system = state.system.lock().unwrap();
     
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    system.processes().len()
}

#[tauri::command]
fn get_network_speed(state: tauri::State<AppState>) -> (u64, u64) {
    
    let mut networks = state.networks.lock().unwrap();
    
    networks.refresh(true); 
    
    let mut bytes_in = 0u64;
    let mut bytes_out = 0u64;
    
    for (_, data) in networks.iter() {
        bytes_in += data.received();
        bytes_out += data.transmitted();
    }
    (bytes_in, bytes_out)
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
    
    let app_state = AppState {
        system: Mutex::new(System::new_all()),
        networks: Mutex::new(Networks::new_with_refreshed_list()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state) 
        .invoke_handler(tauri::generate_handler![
            get_recent_file_os, 
            get_uptime, 
            get_process_count,
            get_network_speed 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}