use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub service_type: String,
    pub script: String,
    pub start: Option<String>,
    pub restart_policy: Option<String>,
    pub restart_delay: Option<u32>,
    pub auto_start: Option<bool>,
    pub user: Option<String>,
    pub env_file: Option<String>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub max_size: Option<String>,
    pub max_files: Option<u32>,
    pub compress: Option<bool>,
}

impl ServiceConfig {
    pub fn load(service_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path(service_name)?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: ServiceConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn config_path(service_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let services_dir = Config::load()?.services_dir()?;
        Ok(services_dir.join(service_name).join("service.toml"))
    }

    pub fn service_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let services_dir = Config::load()?.services_dir()?;
        Ok(services_dir.join(&self.name))
    }

    pub fn script_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let service_dir = self.service_dir()?;
        Ok(service_dir.join(&self.script))
    }

    pub fn env_file_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let service_dir = self.service_dir()?;
        Ok(service_dir.join(".env"))
    }

    pub fn logs_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let service_dir = self.service_dir()?;
        Ok(service_dir.join("logs"))
    }

    pub fn stdout_log(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let logs_dir = self.logs_dir()?;
        let filename = self
            .logging
            .as_ref()
            .and_then(|l| l.stdout.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("stdout.log");
        Ok(logs_dir.join(filename))
    }

    pub fn stderr_log(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let logs_dir = self.logs_dir()?;
        let filename = self
            .logging
            .as_ref()
            .and_then(|l| l.stderr.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("stderr.log");
        Ok(logs_dir.join(filename))
    }
}
