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
     
    let specifics = sysinfo::ProcessRefreshKind::nothing().without_tasks();
   
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, true, specifics);
    
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

#[tauri::command]
fn get_installed_apps() -> Vec<String> {
    
    #[cfg(target_os = "windows")]
    { 
        windows::installed_apps() 
    }

    #[cfg(target_os = "linux")]
    { 
        linux::installed_apps() 
    }

    #[cfg(target_os = "macos")]
    { 
        macos::installed_apps() }
}

#[tauri::command]
fn get_open_connections() -> usize {
    #[cfg(target_os = "windows")]
    {
        windows::get_open_connections()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_open_connections()
    }
    #[cfg(target_os = "macos")]
    {
        macos::get_open_connections()
    }
}

#[tauri::command]
fn get_connected_lan_devices() -> usize {
    #[cfg(target_os = "windows")]
    {
        windows::get_connected_lan_devices()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_connected_lan_devices()
    }
    #[cfg(target_os = "macos")]
    {
        macos::get_connected_lan_devices()
    }
}

#[tauri::command]
fn get_listening_ports() -> usize {
    #[cfg(target_os = "windows")]
    {
        windows::get_listening_ports()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_listening_ports()
    }
    #[cfg(target_os = "macos")]
    {
        macos::get_listening_ports()
    }
}

#[tauri::command]
fn get_os_name() -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_os_name()
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::get_os_name()
    }

    #[cfg(target_os = "macos")]
    {
        String::from("macOS")
    }
}

#[tauri::command]
fn get_cpu_usage(state: tauri::State<AppState>) -> f32 {
    let mut system = state.system.lock().unwrap();
    system.refresh_cpu_usage();
    system.global_cpu_usage()
}

#[tauri::command]
fn get_cpu_speed(state: tauri::State<AppState>) -> u64 {
    let mut system = state.system.lock().unwrap();
    system.refresh_cpu_frequency();

    let cpus = system.cpus();
    if cpus.is_empty() {
        return 0;
    }

    let total: u64 = cpus.iter().map(|c| c.frequency()).sum();
    total / cpus.len() as u64 // average across all logical cores, in MHz
}

#[tauri::command]
fn get_ram_usage(state: tauri::State<AppState>) -> (u64, u64) {
    let mut system = state.system.lock().unwrap();
    system.refresh_memory();
    let used = system.used_memory();
    let total = system.total_memory();
    (used, total)
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
            get_network_speed,
            get_installed_apps,
            get_open_connections,
            get_connected_lan_devices,
            get_os_name,
            get_listening_ports,
            get_cpu_usage,
            get_cpu_speed,
            get_ram_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}