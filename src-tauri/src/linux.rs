use crate::UptimeInfo;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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