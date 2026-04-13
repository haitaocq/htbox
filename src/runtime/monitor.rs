use std::fs;

pub struct ProcessInfo {
    pub pid: u32,
    pub cpu: f32,
    pub memory: u64,
    pub threads: u32,
    pub uptime: u64,
}

pub fn get_process_info(pid: u32) -> Result<ProcessInfo, Box<dyn std::error::Error>> {
    let stat_path = format!("/proc/{}/stat", pid);
    let status_path = format!("/proc/{}/status", pid);

    let stat_content = fs::read_to_string(&stat_path)?;
    let status_content = fs::read_to_string(&status_path)?;

    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    let _utime: u64 = parts.get(13).unwrap_or(&"0").parse().unwrap_or(0);
    let _stime: u64 = parts.get(14).unwrap_or(&"0").parse().unwrap_or(0);
    let starttime: u64 = parts.get(21).unwrap_or(&"0").parse().unwrap_or(0);

    let uptime_str = fs::read_to_string("/proc/uptime")?;
    let uptime: f64 = uptime_str
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    let hertz = 100u64;
    let process_uptime = if starttime > 0 {
        (uptime as u64 * hertz - starttime) / hertz
    } else {
        0
    };

    let mut memory = 0u64;
    let mut threads = 1u32;

    for line in status_content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                memory = parts[1].parse().unwrap_or(0) * 1024;
            }
        }
        if line.starts_with("Threads:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                threads = parts[1].parse().unwrap_or(1);
            }
        }
    }

    Ok(ProcessInfo {
        pid,
        cpu: 0.0,
        memory,
        threads,
        uptime: process_uptime,
    })
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
