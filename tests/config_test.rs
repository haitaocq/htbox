use htbox::config::service::{LoggingConfig, ServiceConfig};
use htbox::config::Config;

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(config.general.is_some());
    assert!(config.backend.is_some());
    assert!(config.logging.is_some());
    assert!(config.env.is_some());
}

#[test]
fn test_config_backend_force_systemd() {
    let config = Config::default();
    let force = config.backend.as_ref().and_then(|b| b.force.clone());
    assert_eq!(force, Some("auto".to_string()));
}

#[test]
fn test_config_user_level_default() {
    let config = Config::default();
    let user_level = config.backend.as_ref().and_then(|b| b.user_level);
    assert_eq!(user_level, Some(false));
}

#[test]
fn test_config_logging_level_default() {
    let config = Config::default();
    let level = config.logging.as_ref().and_then(|l| l.level.clone());
    assert_eq!(level, Some("info".to_string()));
}

#[test]
fn test_service_config_fields() {
    let config = ServiceConfig {
        name: "test-service".to_string(),
        description: Some("Test service".to_string()),
        service_type: "daemon".to_string(),
        script: "script.sh".to_string(),
        start: Some("immediate".to_string()),
        restart_policy: Some("on-failure".to_string()),
        restart_delay: Some(5),
        auto_start: Some(true),
        user: None,
        env_file: None,
        logging: Some(LoggingConfig {
            stdout: Some("logs/stdout.log".to_string()),
            stderr: Some("logs/stderr.log".to_string()),
            max_size: None,
            max_files: None,
            compress: None,
        }),
    };

    assert_eq!(config.name, "test-service");
    assert_eq!(config.service_type, "daemon");
    assert_eq!(config.auto_start, Some(true));
}

#[test]
fn test_service_config_daemon_type() {
    let config = ServiceConfig {
        name: "test".to_string(),
        description: None,
        service_type: "daemon".to_string(),
        script: "script.sh".to_string(),
        start: None,
        restart_policy: None,
        restart_delay: None,
        auto_start: None,
        user: None,
        env_file: None,
        logging: None,
    };

    assert_eq!(config.service_type, "daemon");
}

#[test]
fn test_service_config_onetime_type() {
    let config = ServiceConfig {
        name: "test".to_string(),
        description: None,
        service_type: "onetime".to_string(),
        script: "script.sh".to_string(),
        start: None,
        restart_policy: None,
        restart_delay: None,
        auto_start: None,
        user: None,
        env_file: None,
        logging: None,
    };

    assert_eq!(config.service_type, "onetime");
}

#[test]
fn test_service_config_fields_complete() {
    let config = ServiceConfig {
        name: "my-service".to_string(),
        description: Some("My service description".to_string()),
        service_type: "onetime".to_string(),
        script: "run.sh".to_string(),
        start: Some("manual".to_string()),
        restart_policy: Some("always".to_string()),
        restart_delay: Some(10),
        auto_start: Some(false),
        user: Some("ubuntu".to_string()),
        env_file: Some(".env".to_string()),
        logging: Some(LoggingConfig {
            stdout: Some("output.log".to_string()),
            stderr: Some("error.log".to_string()),
            max_size: Some("100M".to_string()),
            max_files: Some(3),
            compress: Some(true),
        }),
    };

    assert_eq!(config.name, "my-service");
    assert_eq!(
        config.description,
        Some("My service description".to_string())
    );
    assert_eq!(config.service_type, "onetime");
    assert_eq!(config.script, "run.sh");
    assert_eq!(config.start, Some("manual".to_string()));
    assert_eq!(config.restart_policy, Some("always".to_string()));
    assert_eq!(config.restart_delay, Some(10));
    assert_eq!(config.auto_start, Some(false));
    assert_eq!(config.user, Some("ubuntu".to_string()));
    assert_eq!(config.env_file, Some(".env".to_string()));

    let logging = config.logging.unwrap();
    assert_eq!(logging.stdout, Some("output.log".to_string()));
    assert_eq!(logging.stderr, Some("error.log".to_string()));
    assert_eq!(logging.max_size, Some("100M".to_string()));
    assert_eq!(logging.max_files, Some(3));
    assert_eq!(logging.compress, Some(true));
}
