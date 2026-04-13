# 后端实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现后端检测逻辑和 systemd/cron 后端实现

**Architecture:** 根据环境自动检测选择 systemd 或 cron 后端

**Tech Stack:** Rust, std::process::Command

---

## 任务 1: 实现后端检测和 trait 定义

**Files:**
- Modify: `/workspace/htbox/src/backend/mod.rs`

- [ ] **Step 1: 实现后端检测逻辑和 Trait**

```rust
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
                "auto" => {}
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
        .is_err() {
        return Ok(Backend::Cron);
    }
    
    let output = std::process::Command::new("systemctl")
        .args(["list-units", "--type=service", "--state=running", "--no-pager"])
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

pub mod systemd;
pub mod cron;

use systemd::SystemdBackend;
use cron::CronBackend;
```

- [ ] **Step 2: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 2: 实现 Systemd 后端

**Files:**
- Create: `/workspace/htbox/src/backend/systemd.rs`

- [ ] **Step 3: 实现 SystemdBackend**

```rust
use super::{ServiceBackend, ServiceStatus};
use std::path::PathBuf;
use std::process::Command;

pub struct SystemdBackend {
    user_level: bool,
}

impl SystemdBackend {
    pub fn new() -> Self {
        let user_level = std::env::var("HTBOX_USER_LEVEL").unwrap_or_default() == "true";
        SystemdBackend { user_level }
    }
    
    fn systemctl_cmd(&self) -> Command {
        let mut cmd = Command::new("systemctl");
        if self.user_level {
            cmd.arg("--user");
        }
        cmd
    }
    
    fn service_name(&self, name: &str) -> String {
        format!("htbox-{}.service", name)
    }
    
    pub fn generate_unit_file(&self, service_name: &str, service_config: &crate::config::ServiceConfig) -> Result<String, Box<dyn std::error::Error>> {
        let service_dir = service_config.service_dir()?;
        let unit = format!(
            r#"[Unit]
Description=htbox - {}

[Service]
Type=simple
{}
WorkingDirectory={}
EnvironmentFile={}
ExecStart={}
Restart={}
RestartSec={}
StandardOutput=append:{}
StandardError=append:{}

[Install]
WantedBy=multi-user.target
"#,
            service_config.description.as_deref().unwrap_or(service_name),
            service_config.user.as_ref().map(|u| format!("User={}", u)).unwrap_or_default(),
            service_dir.display(),
            service_config.env_file_path()?.display(),
            service_config.script_path()?.display(),
            service_config.restart_policy.as_deref().unwrap_or("on-failure"),
            service_config.restart_delay.unwrap_or(5),
            service_config.stdout_log()?.display(),
            service_config.stderr_log()?.display(),
        );
        Ok(unit)
    }
    
    pub fn install_unit(&self, service_name: &str, service_config: &crate::config::ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
        let unit_content = self.generate_unit_file(service_name, service_config)?;
        
        let unit_path = if self.user_level {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".config"));
            config_dir.join("systemd").join("user").join("htbox-my.service")
        } else {
            PathBuf::from(format!("/etc/systemd/system/htbox-{}.service", service_name))
        };
        
        if let Some(parent) = unit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&unit_path, unit_content)?;
        
        self.systemctl_cmd().arg("daemon-reload").output()?;
        
        Ok(())
    }
}

impl ServiceBackend for SystemdBackend {
    fn start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("start")
            .arg(self.service_name(service_name))
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to start service: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        Ok(())
    }
    
    fn stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("stop")
            .arg(self.service_name(service_name))
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to stop service: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        Ok(())
    }
    
    fn restart(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("restart")
            .arg(self.service_name(service_name))
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to restart service: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        Ok(())
    }
    
    fn enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("enable")
            .arg(self.service_name(service_name))
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to enable service: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        Ok(())
    }
    
    fn disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("disable")
            .arg(self.service_name(service_name))
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to disable service: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        Ok(())
    }
    
    fn status(&self, service_name: &str) -> Result<ServiceStatus, Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("show")
            .arg(self.service_name(service_name))
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut running = false;
        let mut pid = None;
        
        for line in output_str.lines() {
            if line.starts_with("ActiveState=") {
                running = line.contains("active");
            }
            if line.starts_with("MainPID=") {
                let pid_str = line.trim_start_matches("MainPID=");
                if pid_str != "0" {
                    pid = pid_str.parse().ok();
                }
            }
        }
        
        Ok(ServiceStatus { running, pid, enabled: false })
    }
    
    fn is_enabled(&self, service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let output = self.systemctl_cmd()
            .arg("is-enabled")
            .arg(self.service_name(service_name))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).contains("enabled"))
    }
}
```

- [ ] **Step 4: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 3: 实现 Cron 后端

**Files:**
- Create: `/workspace/htbox/src/backend/cron.rs`

- [ ] **Step 5: 实现 CronBackend**

```rust
use super::{ServiceBackend, ServiceStatus};
use std::path::PathBuf;
use std::process::Command;

pub struct CronBackend;

impl CronBackend {
    pub fn new() -> Self {
        CronBackend
    }
    
    fn service_dir(&self, service_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        Ok(config.services_dir()?.join(service_name))
    }
    
    fn pid_file(&self, service_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(self.service_dir(service_name)?.join("run").join("pid"))
    }
    
    fn read_pid(&self, service_name: &str) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let pid_file = self.pid_file(service_name)?;
        if !pid_file.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(pid_file)?;
        let pid = content.trim().parse().ok();
        Ok(pid)
    }
    
    fn is_running(&self, pid: u32) -> bool {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    pub fn daemon_start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let service_dir = self.service_dir(service_name)?;
        let run_dir = service_dir.join("run");
        std::fs::create_dir_all(&run_dir)?;
        
        if let Some(pid) = self.read_pid(service_name)? {
            if self.is_running(pid) {
                return Err(format!("Service {} already running with PID {}", service_name, pid).into());
            }
        }
        
        let logs_dir = service_dir.join("logs");
        std::fs::create_dir_all(&logs_dir)?;
        
        let script_path = service_dir.join("script.sh");
        let stdout_log = logs_dir.join("stdout.log");
        let stderr_log = logs_dir.join("stderr.log");
        
        let mut cmd = Command::new("nohup");
        cmd.arg(&script_path)
            .arg(format!(">> {}", stdout_log.display()))
            .arg(format!("2>> {}", stderr_log.display()))
            .arg("&")
            .current_dir(&service_dir);
        
        let child = cmd.spawn()?;
        let pid = child.id();
        
        let pid_file = self.pid_file(service_name)?;
        std::fs::write(pid_file, pid.to_string())?;
        
        Ok(())
    }
    
    pub fn daemon_stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pid) = self.read_pid(service_name)? {
            if self.is_running(pid) {
                std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output()?;
                
                for _ in 0..10 {
                    if !self.is_running(pid) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                
                if self.is_running(pid) {
                    std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output()?;
                }
            }
        }
        
        let pid_file = self.pid_file(service_name)?;
        if pid_file.exists() {
            std::fs::remove_file(pid_file)?;
        }
        
        Ok(())
    }
    
    pub fn onetime_enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let service_dir = self.service_dir(service_name)?;
        let script_path = service_dir.join("script.sh");
        let logs_dir = service_dir.join("logs");
        
        let stdout_log = logs_dir.join("stdout.log");
        let stderr_log = logs_dir.join("stderr.log");
        
        let output = Command::new("crontab")
            .arg("-l")
            .output()?;
        
        let mut lines = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).lines().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        let marker = format!("# htbox-onetime-{}", service_name);
        if !lines.iter().any(|l| l.contains(&marker)) {
            let job = format!(
                "@reboot {} >> {} 2>> {}",
                script_path.display(),
                stdout_log.display(),
                stderr_log.display()
            );
            lines.push(marker);
            lines.push(job);
        }
        
        let mut input = std::process::Stdio::pipe();
        let mut child = Command::new("crontab")
            .arg("-")
            .stdin(input)
            .spawn()?;
        
        use std::io::Write;
        let mut input = child.stdin.take().unwrap();
        for line in &lines {
            writeln!(input, "{}", line)?;
        }
        drop(input);
        child.wait()?;
        
        Ok(())
    }
    
    pub fn onetime_disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("crontab")
            .arg("-l")
            .output()?;
        
        let lines: Vec<String> = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| !l.contains(&format!("htbox-onetime-{}", service_name)))
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };
        
        let mut input = std::process::Stdio::pipe();
        let mut child = Command::new("crontab")
            .arg("-")
            .stdin(input)
            .spawn()?;
        
        use std::io::Write;
        let mut input = child.stdin.take().unwrap();
        for line in &lines {
            writeln!(input, "{}", line)?;
        }
        drop(input);
        child.wait()?;
        
        Ok(())
    }
}

impl ServiceBackend for CronBackend {
    fn start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;
        
        if config.service_type == "daemon" {
            self.daemon_start(service_name)
        } else {
            Ok(())
        }
    }
    
    fn stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;
        
        if config.service_type == "daemon" {
            self.daemon_stop(service_name)
        } else {
            Ok(())
        }
    }
    
    fn restart(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stop(service_name)?;
        self.start(service_name)
    }
    
    fn enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;
        
        if config.service_type == "onetime" {
            self.onetime_enable(service_name)
        } else {
            Ok(())
        }
    }
    
    fn disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;
        
        if config.service_type == "onetime" {
            self.onetime_disable(service_name)
        } else {
            Ok(())
        }
    }
    
    fn status(&self, service_name: &str) -> Result<ServiceStatus, Box<dyn std::error::Error>> {
        let pid = self.read_pid(service_name)?;
        let running = pid.map(|p| self.is_running(p)).unwrap_or(false);
        
        Ok(ServiceStatus {
            running,
            pid,
            enabled: false,
        })
    }
    
    fn is_enabled(&self, service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let output = Command::new("crontab")
            .arg("-l")
            .output()?;
        
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            Ok(content.contains(&format!("htbox-onetime-{}", service_name)))
        } else {
            Ok(false)
        }
    }
}
```

- [ ] **Step 6: 验证编译**

Run: `cd /workspace/htbox && cargo build`

- [ ] **Step 7: 提交**

```bash
git add src/backend/
git commit -m "feat: implement backend (systemd + cron)"
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

- F4.1: 后端自动检测
- F4.2: 容器环境检测
- F4.3: Systemd unit 文件生成
- F4.4: Systemd 基础操作
- F4.5: Cron daemon 管理
- F4.6: Cron onetime 管理
- F4.7: Cron crontab 管理