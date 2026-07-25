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
    #[cfg(target_os = "windows")]
    pub disk_io: Mutex<crate::windows::DiskIoQuery>,
    #[cfg(not(target_os = "windows"))]
    pub last_disk_io: Mutex<Option<(u64, u64)>>,
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
fn get_disk_io(state: tauri::State<AppState>) -> (u64, u64) {
    #[cfg(target_os = "windows")]
    {
        let mut disk_io = state.disk_io.lock().unwrap();
        windows::poll_disk_io(&mut disk_io)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let (current_read, current_written) = {
            #[cfg(target_os = "linux")]
            { linux::read_disk_totals() }
            #[cfg(target_os = "macos")]
            { macos::read_disk_totals() }
        };

        let mut last = state.last_disk_io.lock().unwrap();
        let delta = match *last {
            Some((prev_read, prev_written)) => (
                current_read.saturating_sub(prev_read),
                current_written.saturating_sub(prev_written),
            ),
            None => (0, 0),
        };
        *last = Some((current_read, current_written));
        
        delta
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
fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_os_version()
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::get_os_version()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_os_version()
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

#[tauri::command]
fn get_connected_peripherals() -> usize {
    #[cfg(target_os = "windows")]
    {
        windows::get_connected_peripherals()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_connected_peripherals()
    }
    #[cfg(target_os = "macos")]
    {
        macos::get_connected_peripherals()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    let app_state = AppState {
        system: Mutex::new(System::new_all()),
        networks: Mutex::new(Networks::new_with_refreshed_list()),
        #[cfg(not(target_os = "windows"))]
        last_disk_io: Mutex::new(None),
        #[cfg(target_os = "windows")]
        disk_io: Mutex::new(windows::init_disk_io_query()),
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
            get_disk_io,
            get_connected_peripherals,
            get_os_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}