use clap::{Parser, Subcommand};

pub mod cmd_config;
pub mod cmd_runner;

use cmd_config::CmdConfig;
use cmd_runner::{execute, parse_args, validate_params};

#[derive(Parser, Debug)]
pub struct CmdCmd {
    #[command(subcommand)]
    pub command: CmdCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CmdCommands {
    List,
    Add {
        name: Option<String>,
        description: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        timeout: Option<u64>,
    },
    Run {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
    Edit {
        name: String,
    },
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
            CmdCommands::Add {
                name,
                description,
                command,
                timeout,
            } => {
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
