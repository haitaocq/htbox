use clap::{Parser, Subcommand};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;

use crate::backend::create_backend;
use crate::config::env::{load_env_file, load_service_env};
use crate::config::{Config, ServiceConfig};

#[derive(Parser, Debug)]
pub struct ServiceCmd {
    #[command(subcommand)]
    pub command: ServiceCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServiceCommands {
    /// 立即启动服务进程
    Start { name: String },
    /// 停止服务进程
    Stop { name: String },
    /// 重启服务
    Restart { name: String },
    /// 设置开机自启
    Enable { name: String },
    /// 取消开机自启
    Disable { name: String },
    /// 查看服务状态
    Status { name: String },
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
    Edit { name: String },
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
    List { name: String },
    /// 添加服务环境变量
    Add {
        name: String,
        key: String,
        value: String,
    },
    /// 删除服务环境变量
    Remove { name: String, key: String },
}

impl ServiceCmd {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            ServiceCommands::Start { name } => start_service(&name),
            ServiceCommands::Stop { name } => stop_service(&name),
            ServiceCommands::Restart { name } => restart_service(&name),
            ServiceCommands::Enable { name } => enable_service(&name),
            ServiceCommands::Disable { name } => disable_service(&name),
            ServiceCommands::Status { name } => show_status(&name),
            ServiceCommands::List => list_services(),
            ServiceCommands::Add { .. } => add_service_interactive(),
            ServiceCommands::Edit { name } => {
                println!("Edit command not yet implemented. Please manually edit the service config file at ~/.htbox/services/{}/service.toml", name);
                Ok(())
            }
            ServiceCommands::Remove { name, force } => remove_service(&name, force),
            ServiceCommands::Logs {
                name,
                n,
                follow,
                errors,
            } => show_logs(&name, n, follow, errors),
            ServiceCommands::Env { command } => match command {
                EnvCommands::List { name } => list_env(&name),
                EnvCommands::Add { name, key, value } => add_env(&name, &key, &value),
                EnvCommands::Remove { name, key } => remove_env(&name, &key),
            },
        }
    }
}

fn list_services() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let services_dir = config.services_dir()?;

    if !services_dir.exists() {
        println!("No services configured yet.");
        return Ok(());
    }

    let entries = std::fs::read_dir(&services_dir)?;
    let mut services: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                services.push(name.to_string());
            }
        }
    }

    services.sort();

    if services.is_empty() {
        println!("No services configured yet.");
    } else {
        println!("Configured services:");
        for name in &services {
            let backend = create_backend();
            match backend.status(name) {
                Ok(status) => {
                    let status_str = if status.running {
                        "[running]"
                    } else {
                        "[stopped]"
                    };
                    let enabled_str = if status.enabled { "[enabled]" } else { "" };
                    println!("  {} {} {}", name, status_str, enabled_str);
                }
                Err(_) => {
                    println!("  {}", name);
                }
            }
        }
    }

    Ok(())
}

fn show_status(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    let status = backend.status(name)?;

    println!("Service: {}", name);
    println!(
        "Status: {}",
        if status.running { "running" } else { "stopped" }
    );

    if let Some(pid) = status.pid {
        println!("PID: {}", pid);
    }

    println!("Enabled: {}", if status.enabled { "yes" } else { "no" });

    if let Ok(config) = ServiceConfig::load(name) {
        if let Some(desc) = config.description {
            println!("Description: {}", desc);
        }
        println!("Type: {}", config.service_type);
        println!("Script: {}", config.script);
    }

    Ok(())
}

fn start_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    backend.start(name)?;
    println!("Service {} started.", name);
    Ok(())
}

fn stop_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    backend.stop(name)?;
    println!("Service {} stopped.", name);
    Ok(())
}

fn restart_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    backend.restart(name)?;
    println!("Service {} restarted.", name);
    Ok(())
}

fn enable_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    backend.enable(name)?;
    println!("Service {} enabled.", name);
    Ok(())
}

fn disable_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();
    backend.disable(name)?;
    println!("Service {} disabled.", name);
    Ok(())
}

fn show_logs(
    name: &str,
    n: usize,
    follow: bool,
    errors: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = match ServiceConfig::load(name) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load service config for '{}': {}", name, e);
            return Err(e);
        }
    };

    let log_file = if errors {
        config.stderr_log()?
    } else {
        config.stdout_log()?
    };

    if !log_file.exists() {
        println!("Log file not found: {}", log_file.display());
        return Ok(());
    }

    if follow {
        show_logs_follow(&log_file, errors)
    } else {
        show_logs_tail(&log_file, n, errors)
    }
}

fn show_logs_tail(
    log_file: &std::path::Path,
    n: usize,
    errors: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(log_file)?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    let start = if all_lines.len() > n {
        all_lines.len() - n
    } else {
        0
    };

    for line in &all_lines[start..] {
        if errors {
            if line.to_lowercase().contains("error")
                || line.to_lowercase().contains("err")
                || line.to_lowercase().contains("fatal")
                || line.to_lowercase().contains("panic")
            {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}

fn show_logs_follow(
    log_file: &std::path::Path,
    errors: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(log_file)?;
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::End(0))?;

    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        if errors {
            if line.to_lowercase().contains("error")
                || line.to_lowercase().contains("err")
                || line.to_lowercase().contains("fatal")
                || line.to_lowercase().contains("panic")
            {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}

fn add_service_interactive() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("Adding new service...");

    print!("Service name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();

    if name.is_empty() {
        return Err("Service name is required".into());
    }

    print!("Description (optional): ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim().to_string();

    print!("Service type (daemon/onetime, default: daemon): ");
    io::stdout().flush()?;
    let mut service_type = String::new();
    io::stdin().read_line(&mut service_type)?;
    let service_type = if service_type.trim().is_empty() {
        "daemon".to_string()
    } else {
        service_type.trim().to_string()
    };

    print!("Script path (default: script.sh): ");
    io::stdout().flush()?;
    let mut script = String::new();
    io::stdin().read_line(&mut script)?;
    let script = if script.trim().is_empty() {
        "script.sh".to_string()
    } else {
        script.trim().to_string()
    };

    print!("Restart policy (on-failure/always/never, default: on-failure): ");
    io::stdout().flush()?;
    let mut restart_policy = String::new();
    io::stdin().read_line(&mut restart_policy)?;
    let restart_policy = if restart_policy.trim().is_empty() {
        "on-failure".to_string()
    } else {
        restart_policy.trim().to_string()
    };

    print!("Auto start (yes/no, default: no): ");
    io::stdout().flush()?;
    let mut auto_start = String::new();
    io::stdin().read_line(&mut auto_start)?;
    let auto_start = auto_start.trim().to_lowercase() == "yes";

    add_service(
        &name,
        if description.is_empty() {
            None
        } else {
            Some(description)
        },
        &service_type,
        &script,
        &restart_policy,
        auto_start,
    )
}

fn add_service(
    name: &str,
    description: Option<String>,
    service_type: &str,
    script: &str,
    restart_policy: &str,
    auto_start: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let services_dir = config.services_dir()?;
    let service_dir = services_dir.join(name);

    std::fs::create_dir_all(&service_dir)?;
    std::fs::create_dir_all(service_dir.join("logs"))?;
    std::fs::create_dir_all(service_dir.join("run"))?;

    let toml_content = format!(
        r#"name = "{}"
description = {}
type = "{}"
script = "{}"
restart_policy = "{}"
auto_start = {}
"#,
        name,
        description
            .map(|d| format!("\"{}\"", d))
            .unwrap_or_else(|| "null".to_string()),
        service_type,
        script,
        restart_policy,
        auto_start
    );

    let config_path = service_dir.join("service.toml");
    std::fs::write(&config_path, toml_content)?;

    let script_path = service_dir.join(script);
    if !script_path.exists() {
        std::fs::write(
            &script_path,
            "#!/bin/bash\n# Your service script here\nsleep infinity\n",
        )?;
        std::fs::set_permissions(&script_path, PermissionsExt::from_mode(0o755))?;
    }

    let env_path = service_dir.join(".env");
    if !env_path.exists() {
        std::fs::write(&env_path, "# Service environment variables\n")?;
    }

    println!("Service '{}' added successfully.", name);
    println!("Configuration: {}", config_path.display());
    println!("Script: {}", script_path.display());

    Ok(())
}

fn remove_service(name: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend();

    if !force {
        let config = Config::load()?;
        let services_dir = config.services_dir()?;
        let service_dir = services_dir.join(name);

        if service_dir.exists() {
            println!("Are you sure you want to remove service '{}'? (y/N)", name);
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("Cancelled.");
                return Ok(());
            }
        }
    }

    if let Ok(status) = backend.status(name) {
        if status.running {
            println!("Stopping service {} before removal...", name);
            backend.stop(name)?;
        }
    }

    if let Ok(config) = Config::load() {
        let services_dir = config.services_dir()?;
        let service_dir = services_dir.join(name);

        if service_dir.exists() {
            std::fs::remove_dir_all(&service_dir)?;
            println!("Service directory removed.");
        }
    }

    println!("Service '{}' removed.", name);

    Ok(())
}

fn list_env(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_vars = load_service_env(name)?;

    if env_vars.is_empty() {
        println!("No environment variables set for service '{}'.", name);
        return Ok(());
    }

    println!("Environment variables for service '{}':", name);
    for (key, value) in env_vars.iter() {
        println!("  {}={}", key, value);
    }

    Ok(())
}

fn add_env(name: &str, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service_config = ServiceConfig::load(name)?;
    let env_path = service_config.env_file_path()?;

    std::fs::create_dir_all(env_path.parent().unwrap())?;

    let mut vars = load_env_file(&env_path)?;
    vars.insert(key.to_string(), value.to_string());

    let mut content = String::new();
    content.push_str("# Service environment variables\n");
    for (k, v) in vars.iter() {
        content.push_str(&format!("{}={}\n", k, v));
    }

    std::fs::write(&env_path, content)?;

    println!("Added {}={} to service '{}'.", key, value, name);

    Ok(())
}

fn remove_env(name: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service_config = ServiceConfig::load(name)?;
    let env_path = service_config.env_file_path()?;

    if !env_path.exists() {
        return Err(format!("No environment file found for service '{}'.", name).into());
    }

    let mut vars = load_env_file(&env_path)?;

    if !vars.contains_key(key) {
        return Err(format!("Key '{}' not found in service '{}'.", key, name).into());
    }

    vars.remove(key);

    let mut content = String::new();
    content.push_str("# Service environment variables\n");
    for (k, v) in vars.iter() {
        content.push_str(&format!("{}={}\n", k, v));
    }

    std::fs::write(&env_path, content)?;

    println!("Removed {} from service '{}'.", key, name);

    Ok(())
}
