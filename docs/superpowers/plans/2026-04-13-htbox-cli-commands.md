# CLI 命令结构实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的 CLI 命令结构，包括 service 和 cmd 子命令

**Architecture:** 使用 clap derive 宏定义 CLI 结构，实现所有子命令

**Tech Stack:** Rust, clap, thiserror, anyhow

---

## 任务 1: 实现 CLI 入口和服务子命令

**Files:**
- Modify: `/workspace/htbox/src/cli.rs`
- Modify: `/workspace/htbox/src/commands.rs`
- Create: `/workspace/htbox/src/commands/service.rs`

- [ ] **Step 1: 实现 service 子命令结构**

在 `commands/service.rs` 中定义：

```rust
use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser, Debug)]
pub struct ServiceCmd {
    #[command(subcommand)]
    pub command: ServiceCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServiceCommands {
    /// 立即启动服务进程
    Start {
        name: String,
    },
    /// 停止服务进程
    Stop {
        name: String,
    },
    /// 重启服务
    Restart {
        name: String,
    },
    /// 设置开机自启
    Enable {
        name: String,
    },
    /// 取消开机自启
    Disable {
        name: String,
    },
    /// 查看服务状态
    Status {
        name: String,
    },
    /// 列出所有服务
    List,
    /// 添加服务 (交互式/参数)
    Add {
        name: Option<String>,
        description: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        script: Option<String>,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        restart_policy: Option<String>,
        #[arg(long)]
        auto_start: Option<bool>,
    },
    /// 编辑服务配置
    Edit {
        name: String,
    },
    /// 删除服务
    Remove {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    /// 查看服务日志
    Logs {
        name: String,
        #[arg(short, long, default_value = "50")]
        n: usize,
        #[arg(short, long)]
        follow: bool,
        #[arg(long)]
        errors: bool,
    },
    /// 服务环境变量管理
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum EnvCommands {
    /// 列出服务环境变量
    List {
        name: String,
    },
    /// 添加服务环境变量
    Add {
        name: String,
        key: String,
        value: String,
    },
    /// 删除服务环境变量
    Remove {
        name: String,
        key: String,
    },
}
```

- [ ] **Step 2: 实现 service 命令执行逻辑**

```rust
impl ServiceCmd {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            ServiceCommands::Start { name } => {
                println!("Starting service: {}", name);
                Ok(())
            }
            ServiceCommands::Stop { name } => {
                println!("Stopping service: {}", name);
                Ok(())
            }
            ServiceCommands::Restart { name } => {
                println!("Restarting service: {}", name);
                Ok(())
            }
            ServiceCommands::Enable { name } => {
                println!("Enabling service: {}", name);
                Ok(())
            }
            ServiceCommands::Disable { name } => {
                println!("Disabling service: {}", name);
                Ok(())
            }
            ServiceCommands::Status { name } => {
                println!("Service status: {}", name);
                Ok(())
            }
            ServiceCommands::List => {
                println!("Listing services...");
                Ok(())
            }
            ServiceCommands::Add { .. } => {
                println!("Adding service...");
                Ok(())
            }
            ServiceCommands::Edit { name } => {
                println!("Editing service: {}", name);
                Ok(())
            }
            ServiceCommands::Remove { name, force } => {
                println!("Removing service: {} (force: {})", name, force);
                Ok(())
            }
            ServiceCommands::Logs { name, n, follow, errors } => {
                println!("Logs for {}: lines={}, follow={}, errors={}", name, n, follow, errors);
                Ok(())
            }
            ServiceCommands::Env { command } => {
                match command {
                    EnvCommands::List { name } => println!("Env list: {}", name),
                    EnvCommands::Add { name, key, value } => println!("Env add: {} {}={}", name, key, value),
                    EnvCommands::Remove { name, key } => println!("Env remove: {} {}", name, key),
                }
                Ok(())
            }
        }
    }
}
```

- [ ] **Step 3: 更新 commands.rs 导出 service 模块**

```rust
pub mod service;

pub use service::ServiceCmd;
```

- [ ] **Step 4: 更新 cli.rs 使用 ServiceCmd**

```rust
use clap::{Parser, Subcommand};
use commands::ServiceCmd;

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
    Service(ServiceCmd),
    Cmd(CmdCmd),
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Service(cmd) => cmd.run(),
        Commands::Cmd(cmd) => cmd.run(),
    }
}
```

- [ ] **Step 5: 验证编译**

Run: `cd /workspace/htbox && cargo build`
Expected: 编译成功

- [ ] **Step 6: 测试 --help**

Run: `cd /workspace/htbox && cargo run -- --help`
Expected: 显示帮助信息

---

## 任务 2: 实现 cmd 子命令

**Files:**
- Create: `/workspace/htbox/src/commands/cmd.rs`
- Modify: `/workspace/htbox/src/commands.rs`

- [ ] **Step 7: 实现 cmd 子命令结构**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct CmdCmd {
    #[command(subcommand)]
    pub command: CmdCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CmdCommands {
    /// 列出所有快捷命令
    List,
    /// 添加快捷命令
    Add {
        name: Option<String>,
        description: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// 执行快捷命令
    Run {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// 编辑快捷命令
    Edit {
        name: String,
    },
    /// 删除快捷命令
    Remove {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
}

impl CmdCmd {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            CmdCommands::List => {
                println!("Listing commands...");
                Ok(())
            }
            CmdCommands::Add { .. } => {
                println!("Adding command...");
                Ok(())
            }
            CmdCommands::Run { name, args } => {
                println!("Running command: {} with args: {:?}", name, args);
                Ok(())
            }
            CmdCommands::Edit { name } => {
                println!("Editing command: {}", name);
                Ok(())
            }
            CmdCommands::Remove { name, force } => {
                println!("Removing command: {} (force: {})", name, force);
                Ok(())
            }
        }
    }
}
```

- [ ] **Step 8: 更新 commands.rs 导出 cmd 模块**

```rust
pub mod service;
pub mod cmd;

pub use service::ServiceCmd;
pub use cmd::CmdCmd;
```

- [ ] **Step 9: 验证编译**

Run: `cd /workspace/htbox && cargo build`
Expected: 编译成功

- [ ] **Step 10: 测试 CLI 功能**

```bash
cargo run -- --help
cargo run -- service --help
cargo run -- cmd --help
cargo run -- service list
cargo run -- cmd run test --arg1 --arg2
```

---

## 验证

完成后验证：

```bash
cd /workspace/htbox
cargo build
cargo run -- --help
cargo run -- service --help
cargo run -- cmd --help
cargo run -- service list
```

---

## 关联功能点

- F5.1: CLI 入口定义 (clap derive)
- F5.2: help/version 支持
- F6.1-F6.14: 服务管理命令
- F7.1-F7.5: 快捷命令管理