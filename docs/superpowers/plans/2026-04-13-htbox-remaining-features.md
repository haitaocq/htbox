# 剩余功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现剩余功能：资源监控、状态管理、交互式创建增强

---

## 任务 1: 实现资源监控

**Files:**
- Create: `/workspace/htbox/src/runtime/monitor.rs`

- [ ] **Step 1: 实现资源监控**

```rust
use std::fs;

pub struct ProcessInfo {
    pub pid: u32,
    pub cpu: f32,
    pub memory: u64,
    pub threads: u32,
    pub uptime: u64,
}

pub fn get_process_info(pid: u32) -> Result<ProcessInfo, Box<dyn std::error::Error>> {
    let stat_path = format!("/proc/{}/stat", pid);
    let status_path = format!("/proc/{}/status", pid);
    
    let stat_content = fs::read_to_string(&stat_path)?;
    let status_content = fs::read_to_string(&status_path)?;
    
    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    let utime: u64 = parts.get(13).unwrap_or(&"0").parse().unwrap_or(0);
    let stime: u64 = parts.get(14).unwrap_or(&"0").parse().unwrap_or(0);
    let starttime: u64 = parts.get(21).unwrap_or(&"0").parse().unwrap_or(0);
    
    let uptime = fs::read_to_string("/proc/uptime")?
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0) as u64;
    
    let clock_ticks = 100u64;
    let process_start = starttime / clock_ticks;
    let process_uptime = if process_start > 0 && process_start < uptime {
        uptime - process_start
    } else {
        0
    };
    
    let mut memory = 0u64;
    let mut threads = 0u32;
    
    for line in status_content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                memory = parts[1].parse().unwrap_or(0) * 1024;
            }
        }
        if line.starts_with("Threads:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                threads = parts[1].parse().unwrap_or(1);
            }
        }
    }
    
    let cpu = calculate_cpu_usage(pid);
    
    Ok(ProcessInfo {
        pid,
        cpu,
        memory,
        threads,
        uptime: process_uptime,
    })
}

fn calculate_cpu_usage(pid: u32) -> f32 {
    let stat_path = format!("/proc/{}/stat", pid);
    if let Ok(content) = fs::read_to_string(&stat_path) {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 16 {
            let utime: u64 = parts.get(13).unwrap_or(&"0").parse().unwrap_or(0);
            let stime: u64 = parts.get(14).unwrap_or(&"0").parse().unwrap_or(0);
            let total_time = utime + stime;
            
            if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
                if let Some(uptime) = uptime_str.split_whitespace().next() {
                    if let Ok(uptime_sec) = uptime.parse::<f64>() {
                        let hertz = 100f64;
                        let seconds = uptime_sec;
                        return ((total_time as f64 / hertz) / seconds * 100.0) as f32;
                    }
                }
            }
        }
    }
    0.0
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
```

- [ ] **Step 2: 更新 runtime/mod.rs**

```rust
pub mod monitor;

pub use monitor::{ProcessInfo, get_process_info, format_bytes, format_uptime};
```

- [ ] **Step 3: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 2: 实现状态管理

**Files:**
- Modify: `/workspace/htbox/src/state/json.rs`

- [ ] **Step 4: 实现 state.json 管理**

```rust
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
    
    pub fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        Ok(config.htbox_dir()?.join("state.json"))
    }
    
    pub fn update_service(&mut self, name: &str, state: ServiceState) {
        self.services.insert(name.to_string(), state);
    }
    
    pub fn remove_service(&mut self, name: &str) {
        self.services.remove(name);
    }
    
    pub fn get_service(&self, name: &str) -> Option<&ServiceState> {
        self.services.get(name)
    }
    
    pub fn list_services(&self) -> Vec<&ServiceState> {
        self.services.values().collect()
    }
}
```

- [ ] **Step 5: 更新 state/mod.rs**

```rust
pub mod json;

pub use json::{State, ServiceState};
```

- [ ] **Step 6: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 3: 更新服务状态显示

- [ ] **Step 7: 在 show_status 中添加资源信息**

更新 `commands/service.rs` 中的 `show_status` 函数，添加资源监控信息。

- [ ] **Step 8: 验证编译**

Run: `cd /workspace/htbox && cargo build`

- [ ] **Step 9: 提交**

```bash
git add src/runtime/ src/state/ src/commands/service.rs
git commit -m "feat: add resource monitoring and state management"
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

- F8.1: 状态管理
- F8.2: PID 文件管理
- F8.3: 资源查询
- F8.4: 运行时间计算
- F9.1: 脚本模板生成
- F9.2: 交互式创建
- F9.3: 日志实时跟踪
- F9.4: 错误日志过滤