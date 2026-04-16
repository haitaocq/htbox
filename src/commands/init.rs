use crate::config::global::{BackendConfig, Config, EnvConfig, GeneralConfig, LoggingConfig};
use std::io::{self, Write};

pub fn run_init(reset: bool, force: bool) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::config_path()?;
    let config_exists = config_path.exists();

    if !reset && !force {
        if config_exists {
            if let Ok(config) = Config::load() {
                if config.is_initialized() {
                    println!("htbox 已初始化完成。如需重新配置，请使用 --reset 参数");
                    return Ok(());
                }
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
        general: Some(GeneralConfig {
            user: None,
            work_dir: None,
        }),
        backend: Some(BackendConfig {
            force: Some(backend),
            user_level: Some(user_level),
        }),
        logging: Some(LoggingConfig {
            level: Some(log_level),
            file: None,
        }),
        env: Some(EnvConfig { global_file: None }),
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

    // 更新时区变量
    let global_env_path = htbox_dir.join("envs").join("global.env");
    let content = std::fs::read_to_string(&global_env_path)?;
    let mut new_content = String::new();
    for line in content.lines() {
        if line.starts_with("HTBOX_TIMEZONE=") {
            new_content.push_str(&format!("HTBOX_TIMEZONE={}", timezone));
        } else {
            new_content.push_str(line);
        }
        new_content.push('\n');
    }
    if !content.contains("HTBOX_TIMEZONE=") {
        new_content.push_str(&format!("HTBOX_TIMEZONE={}\n", timezone));
    }
    std::fs::write(&global_env_path, new_content)?;

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
