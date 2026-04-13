use htbox::backend::{Backend, ServiceStatus};

#[test]
fn test_backend_enum_values() {
    let systemd = Backend::Systemd;
    let cron = Backend::Cron;

    assert_ne!(systemd, cron);
}

#[test]
fn test_service_status_fields() {
    let status = ServiceStatus {
        running: true,
        pid: Some(12345),
        enabled: true,
    };

    assert!(status.running);
    assert_eq!(status.pid, Some(12345));
    assert!(status.enabled);
}

#[test]
fn test_service_status_not_running() {
    let status = ServiceStatus {
        running: false,
        pid: None,
        enabled: false,
    };

    assert!(!status.running);
    assert!(status.pid.is_none());
    assert!(!status.enabled);
}

#[test]
fn test_service_status_copy() {
    let status1 = ServiceStatus {
        running: true,
        pid: Some(100),
        enabled: true,
    };

    let status2 = status1.clone();

    assert_eq!(status1.running, status2.running);
    assert_eq!(status1.pid, status2.pid);
    assert_eq!(status1.enabled, status2.enabled);
}
