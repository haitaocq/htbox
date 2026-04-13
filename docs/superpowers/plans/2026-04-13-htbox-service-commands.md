# 服务命令实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现服务命令的完整功能，包括 start/stop/restart/enable/disable/status/list/add/remove/logs/env

**Tech Stack:** Rust, backend 模块, config 模块

---

## 任务 1: 实现服务列表和状态

**Files:**
- Modify: `/workspace/htbox/src/commands/service.rs`

- [ ] **Step 1: 实现服务列表**

```rust
pub fn list_services() -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    let config = Config::load()?;
    let services_dir = config.services_dir()?;
    
    if !services_dir.exists() {
        println!("No services found");
        return Ok(());
    }
    
    let backend = crate::backend::create_backend();
    
    for entry in std::fs::read_dir(services_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                let service_toml = path.join("service.toml");
                
                if service_toml.exists() {
                    if let Ok(status) = backend.status(&name) {
                        let running = if status.running { "Running" } else { "Stopped" };
                        let enabled = if status.enabled { "Enabled" } else { "Disabled" };
                        println!("{} - {} ({})", name, running, enabled);
                    } else {
                        println!("{} - Unknown", name);
                    }
                }
            }
        }
    }
    
    Ok(())
}
```

- [ ] **Step 2: 实现服务状态**

```rust
pub fn show_status(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    use crate::backend::{ServiceStatus, create_backend};
    
    let service_config = ServiceConfig::load(name)?;
    let backend = create_backend();
    let status = backend.status(name)?;
    let enabled = backend.is_enabled(name)?;
    
    println!("Name:        {}", name);
    println!("Type:        {}", service_config.service_type);
    println!("Status:      {}", if status.running { format!("Running (PID: {:?})", status.pid) } else { "Stopped".to_string() });
    println!("Enabled:     {}", if enabled { "Yes" } else { "No" });
    
    println!("\nConfig:");
    println!("  Script:    {}", service_config.script);
    println!("  Restart:   {:?}", service_config.restart_policy);
    println!("  AutoStart: {:?}", service_config.auto_start);
    
    Ok(())
}
```

- [ ] **Step 3: 实现服务 start/stop/restart**

```rust
pub fn start_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let _config = ServiceConfig::load(name)?;
    
    let backend = crate::backend::create_backend();
    backend.start(name)?;
    println!("Service {} started", name);
    Ok(())
}

pub fn stop_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = crate::backend::create_backend();
    backend.stop(name)?;
    println!("Service {} stopped", name);
    Ok(())
}

pub fn restart_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = crate::backend::create_backend();
    backend.restart(name)?;
    println!("Service {} restarted", name);
    Ok(())
}
```

- [ ] **Step 4: 实现 enable/disable**

```rust
pub fn enable_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let config = ServiceConfig::load(name)?;
    
    let backend = crate::backend::create_backend();
    
    if config.auto_start.unwrap_or(false) {
        backend.enable(name)?;
    }
    
    println!("Service {} enabled", name);
    Ok(())
}

pub fn disable_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = crate::backend::create_backend();
    backend.disable(name)?;
    println!("Service {} disabled", name);
    Ok(())
}
```

- [ ] **Step 5: 实现日志查看**

```rust
pub fn show_logs(name: &str, n: usize, follow: bool, errors: bool) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    
    let service_config = ServiceConfig::load(name)?;
    let log_path = if errors {
        service_config.stderr_log()?
    } else {
        service_config.stdout_log()?
    };
    
    if follow {
        let mut file = std::fs::File::open(&log_path)?;
        use std::io::Seek;
        file.seek(std::io::SeekFrom::End(0))?;
        
        use std::io::BufRead;
        let reader = std::io::BufReader::new(file);
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            print!("{}", line);
        }
    } else {
        let content = std::fs::read_to_string(&log_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        
        for line in &lines[start..] {
            if errors && !line.to_lowercase().contains("error") && !line.to_lowercase().contains("fail") {
                continue;
            }
            println!("{}", line);
        }
    }
    
    Ok(())
}
```

- [ ] **Step 6: 实现服务添加**

```rust
pub fn add_service(
    name: Option<String>,
    description: Option<String>,
    service_type: Option<String>,
    script: Option<String>,
    start: Option<String>,
    restart_policy: Option<String>,
    auto_start: Option<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = name.ok_or("Service name is required")?;
    let service_type = service_type.unwrap_or_else(|| "daemon".to_string());
    let script = script.unwrap_or_else(|| "script.sh".to_string());
    
    use crate::config::{Config, ServiceConfig};
    let global_config = Config::load()?;
    let services_dir = global_config.services_dir()?;
    let service_dir = services_dir.join(&name);
    
    std::fs::create_dir_all(&service_dir)?;
    std::fs::create_dir_all(service_dir.join("logs"))?;
    
    let service_config = ServiceConfig {
        name: name.clone(),
        description,
        service_type,
        script: script.clone(),
        start: start.or_else(|| Some("immediate".to_string())),
        restart_policy: restart_policy.or_else(|| Some("on-failure".to_string())),
        restart_delay: Some(5),
        auto_start,
        user: None,
        env_file: None,
        logging: None,
    };
    
    let config_path = service_dir.join("service.toml");
    let content = toml::to_string_pretty(&service_config)?;
    std::fs::write(&config_path, content)?;
    
    let script_path = service_dir.join(&script);
    let script_content = format!(
        r#"#!/bin/bash
# ==========================================
# {} - {}
# Generated by htbox
# ==========================================

set -e

# 加载环境变量
if [ -f "$(dirname "$0")/.env" ]; then
    source "$(dirname "$0")/.env"
fi

# 继承全局环境变量
if [ -f "$HOME/.htbox/envs/global.env" ]; then
    source "$HOME/.htbox/envs/global.env"
fi

# ========== 在此添加你的业务逻辑 ==========

echo "[$(date)] {} started"
"#,
        name,
        service_config.description.as_deref().unwrap_or(""),
        name
    );
    std::fs::write(&script_path, script_content)?;
    std::fs::set_permissions(&script_path, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
    
    let env_path = service_dir.join(".env");
    std::fs::write(&env_path, format!("# {}\n# 在此添加环境变量\n\n", name))?;
    
    println!("Service {} created successfully", name);
    Ok(())
}
```

- [ ] **Step 7: 实现服务删除**

```rust
pub fn remove_service(name: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !force {
        println!("This will permanently delete service '{}'. Use --force to confirm.", name);
        return Ok(());
    }
    
    let backend = crate::backend::create_backend();
    let _ = backend.stop(name);
    let _ = backend.disable(name);
    
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(name)?;
    let service_dir = service_config.service_dir()?;
    
    if service_dir.exists() {
        std::fs::remove_dir_all(service_dir)?;
    }
    
    println!("Service {} removed", name);
    Ok(())
}
```

- [ ] **Step 8: 实现环境变量管理**

```rust
pub fn list_env(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(name)?;
    let env_path = service_config.env_file_path()?;
    
    if !env_path.exists() {
        println!("No environment variables defined");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(env_path)?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            println!("{}={}", key, value);
        }
    }
    
    Ok(())
}

pub fn add_env(name: &str, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(name)?;
    let env_path = service_config.env_file_path()?;
    
    let mut content = if env_path.exists() {
        std::fs::read_to_string(&env_path)?
    } else {
        String::new()
    };
    
    content.push_str(&format!("{}={}\n", key, value));
    std::fs::write(&env_path, content)?;
    
    println!("Added {}={} to {}", key, value, name);
    Ok(())
}

pub fn remove_env(name: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::ServiceConfig;
    let service_config = ServiceConfig::load(name)?;
    let env_path = service_config.env_file_path()?;
    
    if !env_path.exists() {
        return Err(format!("No environment variables file").into());
    }
    
    let content = std::fs::read_to_string(&env_path)?;
    let new_content: String = content
        .lines()
        .filter(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return true;
            }
            if let Some((k, _)) = line.split_once('=') {
                return k != key;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    std::fs::write(&env_path, new_content)?;
    println!("Removed {} from {}", key, name);
    Ok(())
}
```

- [ ] **Step 9: 更新 service.rs 的 run 方法**

更新 `commands/service.rs` 中的 `run` 方法来调用这些函数。

- [ ] **Step 10: 验证编译**

Run: `cd /workspace/htbox && cargo build`

- [ ] **Step 11: 提交**

```bash
git add src/commands/service.rs
git commit -m "feat: implement service commands (start/stop/restart/enable/disable/status/list/add/remove/logs/env)"
```

---

## 验证

完成后验证：

```bash
cd /workspace/htbox
cargo build
cargo run -- service list
```

---

## 关联功能点

- F6.1: start
- F6.2: stop
- F6.3: restart
- F6.4: enable
- F6.5: disable
- F6.6: status
- F6.7: list
- F6.8: add
- F6.9: edit
- F6.10: remove
- F6.11: logs
- F6.12-F6.14: env list/add/remove