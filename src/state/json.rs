use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub services: HashMap<String, ServiceState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceState {
    pub name: String,
    pub service_type: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub enabled: bool,
    pub last_start: Option<u64>,
    pub last_stop: Option<u64>,
}

impl State {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::path()?;

        if !path.exists() {
            return Ok(State::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let state: State = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let _config = Config::load()?;
        Ok(Config::htbox_dir()?.join("state.json"))
    }

    pub fn update_service(&mut self, name: &str, state: ServiceState) {
        self.services.insert(name.to_string(), state);
    }

    pub fn remove_service(&mut self, name: &str) {
        self.services.remove(name);
    }
}
