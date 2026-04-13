# 快捷命令管理实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现快捷命令管理，包括命令配置、参数解析、模板渲染、执行日志

**Tech Stack:** Rust, tera, serde, toml

---

## 任务 1: 实现快捷命令配置结构

**Files:**
- Create: `/workspace/htbox/src/commands/cmd_config.rs`

- [ ] **Step 1: 实现 CmdConfig 结构**

```rust
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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
```

- [ ] **Step 2: 验证编译**

Run: `cd /workspace/htbox && cargo build`

---

## 任务 2: 实现参数解析和模板渲染

**Files:**
- Create: `/workspace/htbox/src/commands/cmd_runner.rs`

- [ ] **Step 3: 实现参数解析**

```rust
use std::collections::HashMap;

pub fn parse_args(args: &[String]) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with("--") {
            if let Some((key, value)) = arg.trim_start_matches("--").split_once('=') {
                let key = key.replace('-', "_");
                params.insert(key, value.to_string());
            } else {
                let key = arg.trim_start_matches("--").replace('-', "_");
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    params.insert(key, args[i + 1].clone());
                    i += 1;
                } else {
                    params.insert(key, "true".to_string());
                }
            }
        } else if !arg.starts_with('-') {
            // 位置参数暂不处理
        }
        
        i += 1;
    }
    
    Ok(params)
}

pub fn validate_params(
    params: &HashMap<String, String>,
    expected: &std::collections::HashMap<String, crate::commands::cmd_config::ParamDef>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (name, def) in expected {
        if def.required && !params.contains_key(name) {
            if def.default.is_none() {
                return Err(format!("Missing required parameter: {}", name).into());
            }
        }
    }
    
    Ok(())
}
```

- [ ] **Step 4: 实现 Tera 模板渲染**

```rust
use tera::{Tera, Context};

pub fn render_command(command: &str, params: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = Context::new();
    
    for (key, value) in params {
        context.insert(key, value);
    }
    
    let tera = Tera::default();
    let rendered = tera.render_str(command, &context)?;
    Ok(rendered)
}
```

- [ ] **Step 5: 实现命令执行和日志记录**

```rust
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub fn execute(
    config: &crate::commands::cmd_config::CmdConfig,
    params: &HashMap<String, String>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let rendered = render_command(&config.command, params)?;
    
    let log_path = {
        use crate::config::Config;
        let config = Config::load()?;
        config.commands_dir()?.join(format!("{}.log", config.name))
    };
    
    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    
    use std::io::Write;
    writeln!(log_file, "[{}] START: {} {:?}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), config.name, params)?;
    
    let mut parts = rendered.split_whitespace();
    let program = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    
    let mut cmd = Command::new(program);
    cmd.args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let mut child = cmd.spawn()?;
    
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
                writeln!(log_file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line)?;
            }
        }
    }
    
    let status = child.wait()?;
    let exit_code = status.code().unwrap_or(-1);
    
    writeln!(log_file, "[{}] END: {} (exit={})", 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), config.name, exit_code)?;
    
    Ok(exit_code)
}
```

- [ ] **Step 6: 更新 commands/cmd.rs 使用这些功能**

修改 `commands/cmd.rs` 的 `run` 方法来调用实际实现：

```rust
impl CmdCmd {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            CmdCommands::List => {
                let names = CmdConfig::list()?;
                for name in names {
                    if let Ok(config) = CmdConfig::load(&name) {
                        println!("{} - {}", name, config.description.unwrap_or_default());
                    } else {
                        println!("{}", name);
                    }
                }
                Ok(())
            }
            CmdCommands::Add { name, description, command, timeout } => {
                let name = name.ok_or("Name is required")?;
                let command = command.ok_or("Command is required")?;
                
                let config = CmdConfig {
                    name,
                    description,
                    command,
                    timeout,
                    params: std::collections::HashMap::new(),
                    examples: std::collections::HashMap::new(),
                };
                config.save()?;
                println!("Command added successfully");
                Ok(())
            }
            CmdCommands::Run { name, args } => {
                let config = CmdConfig::load(&name)?;
                let params = parse_args(&args)?;
                validate_params(&params, &config.params)?;
                let exit_code = execute(&config, &params)?;
                std::process::exit(exit_code);
            }
            CmdCommands::Edit { name } => {
                println!("Editing command: {} (not implemented)", name);
                Ok(())
            }
            CmdCommands::Remove { name, force } => {
                if !force {
                    println!("Use --force to confirm removal");
                }
                CmdConfig::delete(&name)?;
                println!("Command removed: {}", name);
                Ok(())
            }
        }
    }
}
```

需要在 Cargo.toml 添加 chrono 依赖：

```toml
chrono = "0.4"
```

- [ ] **Step 7: 验证编译**

Run: `cd /workspace/htbox && cargo build`

- [ ] **Step 8: 提交**

```bash
git add src/commands/
git commit -m "feat: implement command management (config, params, template, execution)"
```

---

## 验证

完成后验证：

```bash
cd /workspace/htbox
cargo build
cargo run -- cmd list
```

---

## 关联功能点

- F7.1: list - 列出所有快捷命令
- F7.2: add - 添加快捷命令
- F7.3: run - 执行快捷命令
- F7.4: edit - 编辑快捷命令
- F7.5: remove - 删除快捷命令
- F7.6: 参数解析
- F7.7: 参数校验
- F7.8: Tera 模板渲染
- F7.9: 执行日志