use crate::UptimeInfo;

pub fn get_uptime() -> UptimeInfo {
    UptimeInfo {
        uptime: 0,
        time_system_started: 0,
    }
}

pub fn get_recent_file_macos() -> String {
    "maybe will implement later".to_string()
}