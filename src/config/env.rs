use std::collections::HashMap;
use std::path::Path;

pub fn load_env_file(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(path)?;
    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            vars.insert(key.to_string(), value.to_string());
        }
    }

    Ok(vars)
}

pub fn load_global_env() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use crate::config::Config;
    let config = Config::load()?;
    let global_env_path = config.global_env_file()?;
    load_env_file(&global_env_path)
}

pub fn load_service_env(
    service_name: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(service_name)?;
    let env_path = service_config.env_file_path()?;
    load_env_file(&env_path)
}

pub fn merge_env(
    global: HashMap<String, String>,
    service: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = global;
    merged.extend(service);
    merged
}
