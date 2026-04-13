use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tera::{Context, Tera};

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
        }

        i += 1;
    }

    Ok(params)
}

pub fn validate_params(
    params: &HashMap<String, String>,
    expected: &HashMap<String, super::cmd_config::ParamDef>,
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

pub fn render_command(
    command: &str,
    params: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = Context::new();

    for (key, value) in params {
        context.insert(key, value);
    }

    let mut tera = Tera::default();
    let rendered = tera.render_str(command, &context)?;
    Ok(rendered)
}

pub fn execute(
    config: &super::cmd_config::CmdConfig,
    params: &HashMap<String, String>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let rendered = render_command(&config.command, params)?;

    let log_path = {
        use crate::config::Config;
        let cfg = Config::load()?;
        cfg.commands_dir()?.join("commands.log")
    };

    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    use std::io::Write;
    writeln!(
        log_file,
        "[{}] START: {} {:?}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        config.name,
        params
    )?;

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
                writeln!(
                    log_file,
                    "[{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    line
                )?;
            }
        }
    }

    let status = child.wait()?;
    let exit_code = status.code().unwrap_or(-1);

    writeln!(
        log_file,
        "[{}] END: {} (exit={})",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        config.name,
        exit_code
    )?;

    Ok(exit_code)
}
