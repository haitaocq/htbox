# htbox init 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**目标:** 实现 `htbox init` 交互式初始化命令，配置基本选项并标识初始化状态

**架构:** 在 config/global.rs 中添加 version 字段标识初始化版本，新增 init 命令处理初始化流程，其他命令检查初始化状态

**技术栈:** Rust, clap, dirs crate

---

## 文件变更

| 文件 | 变更 |
|------|------|
| src/config/global.rs | 添加 version 字段, is_initialized() 方法 |
| src/commands/init.rs | 新建: init 命令实现 |
| src/commands/service.rs | 添加 --force 参数, 检查初始化状态 |
| src/cli.rs | 添加 init 子命令 |

---

## 实现步骤

### Task 1: 修改 Config 添加 version 和 is_initialized

**Files:**
- Modify: `src/config/global.rs:1-35`

- [ ] **Step 1: 添加 version 字段到 Config 结构体**

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: Option<String>,  // 新增
    pub general: Option<GeneralConfig>,
    pub backend: Option<BackendConfig>,
    pub logging: Option<LoggingConfig>,
    pub env: Option<EnvConfig>,
}
```

- [ ] **Step 2: 更新 Default 实现**

```rust
impl Default for Config {
    fn default() -> Self {
        Config {
            version: Some("1.0.0".to_string()),
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
```

- [ ] **Step 3: 添加 is_initialized 方法**

```rust
pub fn is_initialized(&self) -> bool {
    self.version.is_some()
}
```

- [ ] **Step 4: 编译验证**

Run: `cargo build --release --target x86_64-unknown-linux-musl`

Expected: SUCCESS

---

### Task 2: 创建 init 命令模块

**Files:**
- Create: `src/commands/init.rs`

- [ ] **Step 1: 创建 init.rs 文件**

```rust
use crate::config::global::Config;
use std::io::{self, Write};

pub fn run_init(reset: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !reset {
        if let Ok(config) = Config::load() {
            if config.is_initialized() {
                println!("htbox 已初始化完成。如需重新配置，请使用 --reset 参数");
                return Ok(());
            }
        }
    }

    println!("欢迎使用 htbox！首次运行将进行初始化配置。\n");

    let htbox_dir = Config::htbox_dir()?;
    println!("htbox 根目录: {}", htbox_dir.display());
    println!();

    // 时区
    print!("时区 (default: Asia/Shanghai): ");
    io::stdout().flush()?;
    let mut timezone = String::new();
    io::stdin().read_line(&mut timezone)?;
    let timezone = timezone.trim();
    let timezone = if timezone.is_empty() {
        "Asia/Shanghai".to_string()
    } else {
        timezone.to_string()
    };

    // 默认后端
    print!("默认后端 (auto/systemd/cron, default: auto): ");
    io::stdout().flush()?;
    let mut backend = String::new();
    io::stdin().read_line(&mut backend)?;
    let backend = backend.trim();
    let backend = if backend.is_empty() {
        "auto".to_string()
    } else {
        backend.to_string()
    };

    // 用户级 systemd
    print!("使用用户级 systemd (yes/no, default: no): ");
    io::stdout().flush()?;
    let mut user_level = String::new();
    io::stdin().read_line(&mut user_level)?;
    let user_level = user_level.trim().to_lowercase() == "yes";

    // 日志级别
    print!("日志级别 (debug/info/warn/error, default: info): ");
    io::stdout().flush()?;
    let mut log_level = String::new();
    io::stdin().read_line(&mut log_level)?;
    let log_level = log_level.trim();
    let log_level = if log_level.is_empty() {
        "info".to_string()
    } else {
        log_level.to_string()
    };

    // 环境变量
    let mut env_vars: Vec<(String, String)> = Vec::new();
    loop {
        print!("是否添加全局环境变量 (yes/no, default: no): ");
        io::stdout().flush()?;
        let mut add_env = String::new();
        io::stdin().read_line(&mut add_env)?;
        
        if add_env.trim().to_lowercase() != "yes" {
            break;
        }

        print!("  KEY: ");
        io::stdout().flush()?;
        let mut key = String::new();
        io::stdin().read_line(&mut key)?;
        let key = key.trim().to_string();

        if key.is_empty() {
            eprintln!("Key 不能为空");
            continue;
        }

        print!("  Value: ");
        io::stdout().flush()?;
        let mut value = String::new();
        io::stdin().read_line(&mut value)?;
        let value = value.trim().to_string();

        env_vars.push((key, value));
    }

    // 创建配置
    let config = Config {
        version: Some("1.0.0".to_string()),
        general: Some(crate::config::global::GeneralConfig {
            user: None,
            work_dir: None,
        }),
        backend: Some(crate::config::global::BackendConfig {
            force: Some(backend),
            user_level: Some(user_level),
        }),
        logging: Some(crate::config::global::LoggingConfig {
            level: Some(log_level),
            file: None,
        }),
        env: Some(crate::config::global::EnvConfig {
            global_file: None,
        }),
    };

    // 确保目录存在
    config.ensure_dirs()?;

    // 添加全局环境变量
    if !env_vars.is_empty() {
        let global_env_path = htbox_dir.join("envs").join("global.env");
        let mut content = std::fs::read_to_string(&global_env_path)?;
        for (key, value) in env_vars {
            content.push_str(&format!("{}={}\n", key, value));
        }
        std::fs::write(&global_env_path, content)?;
    }

    // 添加时区变量
    let global_env_path = htbox_dir.join("envs").join("global.env");
    let content = std::fs::read_to_string(&global_env_path)?;
    if !content.contains("HTBOX_TIMEZONE=") {
        let mut new_content = content.clone();
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&format!("HTBOX_TIMEZONE={}\n", timezone));
        std::fs::write(&global_env_path, new_content)?;
    }

    // 保存配置
    config.save()?;

    println!();
    println!("初始化完成！可用的目录:");
    println!("  - 配置: {}/config.toml", htbox_dir.display());
    println!("  - 环境变量: {}/envs/global.env", htbox_dir.display());
    println!("  - 服务目录: {}/services", htbox_dir.display());
    println!("  - 命令目录: {}/commands", htbox_dir.display());
    println!();
    println!("运行 'htbox service add' 开始创建服务。");

    Ok(())
}
```

- [ ] **Step 2: 在 commands/mod.rs 中导出**

如果文件不存在，创建或修改 `src/commands/mod.rs` 添加:
```rust
pub mod init;
```

---

### Task 3: 添加 init 命令到 CLI

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: 查看现有 CLI 结构**

Run: `cat src/cli.rs | head -60`

- [ ] **Step 2: 添加 init 子命令**

在 Subcommand 枚举中添加:
```rust
Init {
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    force: bool,
},
```

- [ ] **Step 3: 添加命令处理**

在 match 中添加:
```rust
Commands::Init { reset, force } => {
    use crate::commands::init::run_init;
    if reset {
        run_init(true)?;
    } else if force {
        let config = Config::default();
        config.ensure_dirs()?;
        config.save()?;
        println!("已使用默认配置初始化。");
    } else {
        run_init(false)?;
    }
}
```

- [ ] **Step 4: 编译验证**

Run: `cargo build --release --target x86_64-unknown-linux-musl`

Expected: SUCCESS

---

### Task 4: 修改 service 命令添加 --force

**Files:**
- Modify: `src/commands/service.rs`

- [ ] **Step 1: 修改 ServiceCmd 结构体添加 force**

```rust
#[derive(Parser, Debug)]
pub struct ServiceCmd {
    #[arg(long)]
    pub force: bool,
    #[command(subcommand)]
    pub command: ServiceCommands,
}
```

- [ ] **Step 2: 修改 run 方法检查初始化状态**

```rust
impl ServiceCmd {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查初始化状态
        if !self.force {
            let config = Config::load()?;
            if !config.is_initialized() {
                println!("htbox 未初始化，请先运行: htbox init");
                println!("或使用 --force 参数强制使用默认配置");
                return Ok(());
            }
        }
        
        match self.command {
            // ... existing code
        }
    }
}
```

- [ ] **Step 3: 编译验证**

Run: `cargo build --release --target x86_64-unknown-linux-musl`

Expected: SUCCESS

---

### Task 5: 为其他命令添加 --force 支持

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: 添加全局 --force 参数**

在 Commands 枚举的顶层添加:
```rust
#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(long)]
    pub force: bool,
    #[command(subcommand)]
    pub command: Commands,
}
```

- [ ] **Step 2: 修改命令处理传递 force**

修改各命令的 run 方法接受 force 参数，并在内部检查初始化状态

- [ ] **Step 3: 编译验证**

Run: `cargo build --release --target x86_64-unknown-linux-musl`

Expected: SUCCESS

---

### Task 6: 更新设计文档

- [ ] **Step 1: 更新设计文档**

修改 `docs/superpowers/specs/2026-04-13-htbox-cli-rust-design.md` 添加 init 命令说明

---

## 验证

所有任务完成后，运行:
```bash
cargo build --release --target x86_64-unknown-linux-musl
./target/x86_64-unknown-linux-musl/release/htbox init
./target/x86_64-unknown-linux-musl/release/htbox --help
```
