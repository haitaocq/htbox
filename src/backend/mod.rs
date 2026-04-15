use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Systemd,
    Cron,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub force: Option<String>,
    pub user_level: Option<bool>,
}

pub trait ServiceBackend: Send + Sync {
    fn start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn restart(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn status(&self, service_name: &str) -> Result<ServiceStatus, Box<dyn std::error::Error>>;
    fn is_enabled(&self, service_name: &str) -> Result<bool, Box<dyn std::error::Error>>;
    fn install_unit(&self, _service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub enabled: bool,
}

pub fn detect() -> Backend {
    if let Ok(backend) = detect_impl() {
        return backend;
    }
    Backend::Cron
}

fn detect_impl() -> Result<Backend, Box<dyn std::error::Error>> {
    use crate::config::Config;
    let config = Config::load()?;

    if let Some(backend_config) = config.backend.as_ref() {
        if let Some(force) = backend_config.force.as_ref() {
            match force.as_str() {
                "systemd" => return Ok(Backend::Systemd),
                "cron" => return Ok(Backend::Cron),
                _ => {}
            }
        }
    }

    if !Path::new("/run/systemd/system").exists() {
        return Ok(Backend::Cron);
    }

    if std::process::Command::new("systemctl")
        .arg("--version")
        .output()
        .is_err()
    {
        return Ok(Backend::Cron);
    }

    let output = std::process::Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--state=running",
            "--no-pager",
        ])
        .output()?;

    if !output.status.success() || output.stdout.is_empty() {
        return Ok(Backend::Cron);
    }

    if is_container_environment() {
        return Ok(Backend::Cron);
    }

    Ok(Backend::Systemd)
}

fn is_container_environment() -> bool {
    if let Ok(content) = std::fs::read_to_string("/proc/1/cgroup") {
        let indicators = ["/docker/", "/containerd/", "/kubepods/", "containerd"];
        if indicators.iter().any(|i| content.contains(i)) {
            return true;
        }
    }

    Path::new("/.dockerenv").exists()
}

pub fn create_backend() -> Box<dyn ServiceBackend> {
    match detect() {
        Backend::Systemd => Box::new(SystemdBackend::new()),
        Backend::Cron => Box::new(CronBackend::new()),
    }
}

pub mod cron;
pub mod systemd;

use cron::CronBackend;
use systemd::SystemdBackend;
