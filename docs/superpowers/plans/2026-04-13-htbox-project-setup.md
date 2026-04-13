# 项目脚手架搭建实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建 htbox Rust 项目脚手架，配置核心依赖，搭建目录结构

**Architecture:** 使用 cargo 初始化项目，配置 clap 用于 CLI，tokio 用于异步运行时，toml 用于配置解析

**Tech Stack:** Rust, Cargo, clap, tokio, toml, tracing, thiserror, anyhow, tera

---

## 文件结构

根据设计文档第 12 节，项目结构如下：

```
htbox/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── cmd.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── global.rs
│   │   └── service.rs
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── systemd.rs
│   │   └── cron.rs
│   ├── runtime/
│   │   ├── mod.rs
│   │   └── service.rs
│   ├── state/
│   │   ├── mod.rs
│   │   └── json.rs
│   ├── error.rs
│   └── logging.rs
├── tests/
└── examples/
```

---

## 任务 1: 初始化 Rust 项目

**Files:**
- Create: `/workspace/htbox/Cargo.toml`
- Create: `/workspace/htbox/src/main.rs`
- Create: `/workspace/htbox/src/lib.rs`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "htbox"
version = "0.1.0"
edition = "2021"
description = "Linux CLI tool for service and command management"
authors = ["htbox"]
license = "MIT"

[[bin]]
name = "htbox"
path = "src/main.rs"

[lib]
name = "htbox"
path = "src/lib.rs"

[dependencies]
clap = { version = "5.0", features = ["derive", "env"] }
tokio = { version = "1.0", features = ["full"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1.0"
anyhow = "1.0"
tera = "1.0"
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"

[dev-dependencies]
tempfile = "3.0"

[profile.release]
lto = true
codegen-units = 1
```

- [ ] **Step 2: 创建 src/main.rs**

```rust
use htbox::cli;
use htbox::logging;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init()?;
    cli::run()?;
    Ok(())
}
```

- [ ] **Step 3: 创建 src/lib.rs (空模块入口)**

```rust
pub mod cli;
pub mod commands;
pub mod config;
pub mod backend;
pub mod runtime;
pub mod state;
pub mod error;
pub mod logging;
```

- [ ] **Step 4: 验证项目编译**

Run: `cd /workspace/htbox && cargo build`
Expected: 编译成功（会有未定义符号警告，这是正常的）

- [ ] **Step 5: 提交**

```bash
git add Cargo.toml src/main.rs src/lib.rs
git commit -m "feat: init rust project scaffold"
```

---

## 任务 2: 创建基础模块占位文件

**Files:**
- Create: `/workspace/htbox/src/error.rs`
- Create: `/workspace/htbox/src/logging.rs`
- Create: `/workspace/htbox/src/cli.rs`
- Create: `/workspace/htbox/src/commands/mod.rs`
- Create: `/workspace/htbox/src/commands/service.rs`
- Create: `/workspace/htbox/src/commands/cmd.rs`
- Create: `/workspace/htbox/src/config/mod.rs`
- Create: `/workspace/htbox/src/config/global.rs`
- Create: `/workspace/htbox/src/config/service.rs`
- Create: `/workspace/htbox/src/backend/mod.rs`
- Create: `/workspace/htbox/src/backend/systemd.rs`
- Create: `/workspace/htbox/src/backend/cron.rs`
- Create: `/workspace/htbox/src/runtime/mod.rs`
- Create: `/workspace/htbox/src/runtime/service.rs`
- Create: `/workspace/htbox/src/state/mod.rs`
- Create: `/workspace/htbox/src/state/json.rs`

- [ ] **Step 6: 创建 error.rs (错误类型定义)**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Script file not found: {0}")]
    ScriptNotFound(String),
    
    #[error("Permission denied: {0}. Try: sudo htbox ...")]
    PermissionDenied(String),
    
    #[error("Service already running: {0}")]
    ServiceAlreadyRunning(String),
    
    #[error("systemd unavailable, using cron backend")]
    SystemdUnavailable,
    
    #[error("Invalid config: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 7: 创建 logging.rs (日志初始化)**

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("HTBOX_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();
    Ok(())
}
```

- [ ] **Step 8: 创建 cli.rs (CLI 入口占位)**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "htbox")]
#[command(version = "0.1.0")]
#[command(about = "Linux CLI tool for service and command management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Service(service::ServiceCmd),
    Cmd(cmd::CmdCmd),
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Service(cmd) => service::run(cmd),
        Commands::Cmd(cmd) => cmd::run(cmd),
    }
}
```

- [ ] **Step 9: 创建 commands/mod.rs**

```rust
pub mod service;
pub mod cmd;
```

- [ ] **Step 10: 创建 commands/service.rs (占位)**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct ServiceCmd {
    #[command(subcommand)]
    pub command: ServiceCommands,
}

#[derive(Subcommand)]
pub enum ServiceCommands {
    Start { name: String },
    Stop { name: String },
    // ... other variants
}

pub fn run(cmd: ServiceCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.command {
        ServiceCommands::Start { name } => {
            println!("Starting service: {}", name);
            Ok(())
        }
        ServiceCommands::Stop { name } => {
            println!("Stopping service: {}", name);
            Ok(())
        }
        _ => unimplemented!(),
    }
}
```

- [ ] **Step 11: 创建 commands/cmd.rs (占位)**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct CmdCmd {
    #[command(subcommand)]
    pub command: CmdCommands,
}

#[derive(Subcommand)]
pub enum CmdCommands {
    List,
    Add { name: String },
    Run { name: String },
    // ... other variants
}

pub fn run(cmd: CmdCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.command {
        CmdCommands::List => {
            println!("Listing commands...");
            Ok(())
        }
        _ => unimplemented!(),
    }
}
```

- [ ] **Step 12-17: 创建 config 目录模块**

创建以下文件，内容类似：

```rust
// config/mod.rs
pub mod global;
pub mod service;
```

```rust
// config/global.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: Option<GeneralConfig>,
    pub backend: Option<BackendConfig>,
    pub logging: Option<LoggingConfig>,
    pub env: Option<EnvConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub user: Option<String>,
    pub work_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub force: Option<String>,
    pub user_level: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnvConfig {
    pub global_file: Option<String>,
}
```

```rust
// config/service.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,  // daemon or onetime
    pub script: String,
    pub start: Option<String>,
    pub restart_policy: Option<String>,
    pub restart_delay: Option<u32>,
    pub auto_start: Option<bool>,
    pub user: Option<String>,
    pub env_file: Option<String>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub max_size: Option<String>,
    pub max_files: Option<u32>,
    pub compress: Option<bool>,
}
```

- [ ] **Step 18-20: 创建 backend 目录模块**

```rust
// backend/mod.rs
pub mod systemd;
pub mod cron;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Systemd,
    Cron,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub force: Option<String>,
    pub user_level: Option<bool>,
}

pub fn detect() -> Backend {
    // 检测逻辑
    use std::path::Path;
    use std::process::Command;

    // 1. 检查 /run/systemd/system
    if !Path::new("/run/systemd/system").exists() {
        return Backend::Cron;
    }

    // 2. 验证 systemctl 可用
    if Command::new("systemctl").arg("--version").output().is_err() {
        return Backend::Cron;
    }

    // 3. 验证 systemd 运行正常
    let output = Command::new("systemctl")
        .args(["list-units", "--type=service", "--state=running", "--no-pager"])
        .output();
    
    match output {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {}
        _ => return Backend::Cron,
    }

    // 4. 检测容器环境
    if is_container_environment() {
        return Backend::Cron;
    }

    Backend::Systemd
}

fn is_container_environment() -> bool {
    use std::path::Path;
    
    if let Ok(content) = std::fs::read_to_string("/proc/1/cgroup") {
        let indicators = ["/docker/", "/containerd/", "/kubepods/", "containerd"];
        if indicators.iter().any(|i| content.contains(i)) {
            return true;
        }
    }
    
    Path::new("/.dockerenv").exists()
}
```

```rust
// backend/systemd.rs
// systemd 后端实现占位
```

```rust
// backend/cron.rs
// cron 后端实现占位
```

- [ ] **Step 21-23: 创建 runtime 目录模块**

```rust
// runtime/mod.rs
pub mod service;
```

```rust
// runtime/service.rs
// 服务运行管理占位
```

- [ ] **Step 24-26: 创建 state 目录模块**

```rust
// state/mod.rs
pub mod json;
```

```rust
// state/json.rs
// state.json 操作占位
```

- [ ] **Step 27: 验证项目编译**

Run: `cd /workspace/htbox && cargo build`
Expected: 编译成功

- [ ] **Step 28: 提交**

```bash
git add src/
git commit -m "feat: add all module placeholder files"
```

---

## 验证

完成上述任务后，执行以下验证：

```bash
cd /workspace/htbox
cargo build
cargo run -- --help
```

预期输出：
```
Linux CLI tool for service and command management

Usage: htbox [COMMAND]

Commands:
  service  Service management
  cmd      Command management
  help     Print this message or the help of the given subcommand(s)
```

---

## 关联功能点

- F1.1: 创建 Rust 项目脚手架
- F1.2: 配置核心依赖
- F1.3: 项目目录结构搭建
- F2.1: 定义 Error 枚举类型
- F5.1: CLI 入口定义
- F5.2: help/version 支持