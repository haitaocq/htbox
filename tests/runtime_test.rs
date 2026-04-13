use htbox::runtime::{format_bytes, format_uptime, ProcessInfo};

#[test]
fn test_format_bytes_b() {
    assert_eq!(format_bytes(0), "0B");
    assert_eq!(format_bytes(512), "512B");
}

#[test]
fn test_format_bytes_kb() {
    assert_eq!(format_bytes(1024), "1.0KB");
    assert_eq!(format_bytes(1024 * 512), "512.0KB");
}

#[test]
fn test_format_bytes_mb() {
    assert_eq!(format_bytes(1024 * 1024), "1.0MB");
    assert_eq!(format_bytes(1024 * 1024 * 50), "50.0MB");
}

#[test]
fn test_format_bytes_gb() {
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.0GB");
}

#[test]
fn test_format_uptime_minutes() {
    assert_eq!(format_uptime(0), "0m");
    assert_eq!(format_uptime(60), "1m");
    assert_eq!(format_uptime(300), "5m");
    assert_eq!(format_uptime(3540), "59m");
}

#[test]
fn test_format_uptime_hours() {
    assert_eq!(format_uptime(3600), "1h 0m");
    assert_eq!(format_uptime(7200), "2h 0m");
    assert_eq!(format_uptime(3660), "1h 1m");
    assert_eq!(format_uptime(86340), "23h 59m");
}

#[test]
fn test_format_uptime_days() {
    assert_eq!(format_uptime(86400), "1d 0h");
    assert_eq!(format_uptime(172800), "2d 0h");
    assert_eq!(format_uptime(90000), "1d 1h");
    assert_eq!(format_uptime(169200), "1d 23h");
}

#[test]
fn test_process_info_fields() {
    let info = ProcessInfo {
        pid: 12345,
        cpu: 25.5,
        memory: 1024 * 1024 * 100,
        threads: 4,
        uptime: 3600,
    };

    assert_eq!(info.pid, 12345);
    assert!((info.cpu - 25.5).abs() < 0.1);
    assert!(info.memory > 0);
    assert_eq!(info.threads, 4);
    assert_eq!(info.uptime, 3600);
}

#[test]
fn test_process_info_zero() {
    let info = ProcessInfo {
        pid: 0,
        cpu: 0.0,
        memory: 0,
        threads: 1,
        uptime: 0,
    };

    assert_eq!(info.pid, 0);
    assert_eq!(info.cpu, 0.0);
    assert_eq!(info.memory, 0);
    assert_eq!(info.threads, 1);
    assert_eq!(info.uptime, 0);
}
