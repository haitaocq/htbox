use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub general: Option<GeneralConfig>,
    pub backend: Option<BackendConfig>,
    pub logging: Option<LoggingConfig>,
    pub env: Option<EnvConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub user: Option<String>,
    pub work_dir: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub force: Option<String>,
    pub user_level: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvConfig {
    pub global_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
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
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
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
