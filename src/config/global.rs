use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: Option<String>,
    pub general: Option<GeneralConfig>,
    pub backend: Option<BackendConfig>,
    pub logging: Option<LoggingConfig>,
    pub env: Option<EnvConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub user: Option<String>,
    pub work_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BackendConfig {
    pub force: Option<String>,
    pub user_level: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnvConfig {
    pub global_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: Some("1.0.0".to_string()),
            general: Some(GeneralConfig {
                user: None,
                work_dir: None,
            }),
            backend: Some(BackendConfig {
                force: Some("auto".to_string()),
                user_level: Some(false),
            }),
            logging: Some(LoggingConfig {
                level: Some("info".to_string()),
                file: None,
            }),
            env: Some(EnvConfig { global_file: None }),
        }
    }
}

impl Config {
    pub fn is_initialized(&self) -> bool {
        self.version.is_some()
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Err("htbox 未初始化，请先运行 htbox init".into());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let config = Config::default();
            config.ensure_dirs()?;
            config.save()?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    pub fn ensure_dirs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let htbox_dir = Self::htbox_dir()?;

        std::fs::create_dir_all(htbox_dir.join("envs"))?;
        std::fs::create_dir_all(htbox_dir.join("services"))?;
        std::fs::create_dir_all(htbox_dir.join("commands"))?;
        std::fs::create_dir_all(htbox_dir.join("logs"))?;

        let global_env_path = htbox_dir.join("envs").join("global.env");
        if !global_env_path.exists() {
            let default_path = "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin";
            let home = dirs::home_dir()
                .map(|h| h.display().to_string())
                .unwrap_or_else(|| "/root".to_string());
            std::fs::write(
                global_env_path,
                format!(
                    "# ==========================================\n\
# Global Environment Variables\n\
# ==========================================\n\
# This file contains environment variables shared by all services\n\
# Format: KEY=VALUE (shell format)\n\
#\n\
# Built-in variables\n\
HTBOX_HOME={}\n\
HTBOX_LOG_DIR={}/logs\n\
HTBOX_SERVICES_DIR={}/services\n\
HTBOX_TIMEZONE=Asia/Shanghai\n\
HOME={}\n\
PATH={}\n\
#\n\
# User-defined variables\n\
# LOG_DIR=/var/log/htbox\n\
# DATA_DIR=/var/lib/htbox\n\
",
                    htbox_dir.display(),
                    htbox_dir.display(),
                    htbox_dir.display(),
                    home,
                    default_path
                ),
            )?;
        }

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        Ok(home.join(".htbox").join("config.toml"))
    }

    pub fn htbox_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        Ok(home.join(".htbox"))
    }

    pub fn services_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = self
            .general
            .as_ref()
            .and_then(|g| g.work_dir.as_ref())
            .map(|d| PathBuf::from(d))
            .unwrap_or_else(|| Self::htbox_dir().unwrap_or_else(|_| PathBuf::from("")));
        Ok(base.join("services"))
    }

    pub fn global_env_file(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = Self::htbox_dir()?;
        let env_dir = base.join("envs");

        if let Some(env_config) = self.env.as_ref() {
            if let Some(file) = env_config.global_file.as_ref() {
                return Ok(PathBuf::from(file));
            }
        }

        Ok(env_dir.join("global.env"))
    }

    pub fn commands_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = self
            .general
            .as_ref()
            .and_then(|g| g.work_dir.as_ref())
            .map(|d| PathBuf::from(d))
            .unwrap_or_else(|| Self::htbox_dir().unwrap_or_else(|_| PathBuf::from("")));
        Ok(base.join("commands"))
    }
}
