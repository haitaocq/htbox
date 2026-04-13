use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CmdConfig {
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub timeout: Option<u64>,
    #[serde(default)]
    pub params: HashMap<String, ParamDef>,
    #[serde(default)]
    pub examples: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamDef {
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    pub description: Option<String>,
}

impl CmdConfig {
    pub fn load(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path(name)?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: CmdConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn config_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        Ok(config.commands_dir()?.join(format!("{}.toml", name)))
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        let commands_dir = config.commands_dir()?;
        std::fs::create_dir_all(&commands_dir)?;

        let path = commands_dir.join(format!("{}.toml", self.name));
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        let commands_dir = config.commands_dir()?;

        if !commands_dir.exists() {
            return Ok(vec![]);
        }

        let mut names = vec![];
        for entry in std::fs::read_dir(commands_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
            }
        }
        Ok(names)
    }

    pub fn delete(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path(name)?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
