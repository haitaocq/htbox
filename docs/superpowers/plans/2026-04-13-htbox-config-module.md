# 配置管理模块实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现配置管理模块，包括全局配置、服务配置、环境变量加载

**Architecture:** 使用 toml 解析配置，serdeDeserialize 反序列化

**Tech Stack:** Rust, toml, serde, dirs

---

## 任务 1: 实现全局配置模块

**Files:**
- Modify: `/workspace/htbox/src/config/global.rs`

- [ ] **Step 1: 实现全局配置结构体**

```rust
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
            env: Some(EnvConfig {
                global_file: None,
            }),
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
        let base = self.general.as_ref()
            .and_then(|g| g.work_dir.as_ref())
            .map(|d| PathBuf::from(d))
            .unwrap_or_else(|| Self::htbox_dir()?);
        Ok(base.join("services"))
    }
    
    pub fn commands_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = self.general.as_ref()
            .and_then(|g| g.work_dir.as_ref())
            .map(|d| PathBuf::from(d))
            .unwrap_or_else(|| Self::htbox_dir()?);
        Ok(base.join("commands"))
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
}
```

- [ ] **Step 2: 更新 config/mod.rs 导出**

```rust
pub mod global;
pub use global::Config;
```

- [ ] **Step 3: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 2: 实现服务配置模块

**Files:**
- Modify: `/workspace/htbox/src/config/service.rs`

- [ ] **Step 4: 实现服务配置结构体**

```rust
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
        let services_dir = Config::new()?.services_dir()?;
        Ok(services_dir.join(service_name).join("service.toml"))
    }
    
    pub fn service_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let services_dir = Config::new()?.services_dir()?;
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
        let filename = self.logging.as_ref()
            .and_then(|l| l.stdout.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("stdout.log");
        Ok(logs_dir.join(filename))
    }
    
    pub fn stderr_log(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let logs_dir = self.logs_dir()?;
        let filename = self.logging.as_ref()
            .and_then(|l| l.stderr.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("stderr.log");
        Ok(logs_dir.join(filename))
    }
}
```

- [ ] **Step 5: 更新 config/mod.rs 导出 ServiceConfig**

```rust
pub mod global;
pub mod service;

pub use global::Config;
pub use service::ServiceConfig;
```

- [ ] **Step 6: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 3: 实现环境变量加载

**Files:**
- Create: `/workspace/htbox/src/config/env.rs`
- Modify: `/workspace/htbox/src/config/mod.rs`

- [ ] **Step 7: 实现环境变量加载**

```rust
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

pub fn load_service_env(service_name: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(service_name)?;
    let env_path = service_config.env_file_path()?;
    load_env_file(&env_path)
}

pub fn merge_env(global: HashMap<String, String>, service: HashMap<String, String>) -> HashMap<String, String> {
    let mut merged = global;
    merged.extend(service);
    merged
}
```

- [ ] **Step 8: 更新 config/mod.rs 导出**

```rust
pub mod global;
pub mod service;
pub mod env;

pub use global::Config;
pub use service::ServiceConfig;
pub use env::*;
```

- [ ] **Step 9: 验证编译**

Run: `cd /workspace/htbox && cargo build`

- [ ] **Step 10: 提交**

```bash
git add src/config/
git commit -m "feat: implement config management module"
```

---

## 验证

完成后验证：

```bash
cd /workspace/htbox
cargo build
```

---

## 关联功能点

- F3.1: 全局配置解析
- F3.2: 服务配置解析
- F3.3: 全局环境变量加载
- F3.4: 服务环境变量加载