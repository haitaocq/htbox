use crate::commands::{run_init, CmdCmd, ServiceCmd};
use crate::config::global::Config;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "htbox")]
#[command(version = "0.1.0")]
#[command(about = "Linux CLI tool for service and command management")]
pub struct Cli {
    #[arg(long)]
    pub force: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Service(ServiceCmd),
    Cmd(CmdCmd),
    Init {
        #[arg(long)]
        reset: bool,
    },
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 检查初始化状态
    if !cli.force {
        if let Ok(config) = Config::load() {
            if !config.is_initialized() {
                match &cli.command {
                    Commands::Init { .. } => {}
                    _ => {
                        println!("htbox 未初始化，请先运行: htbox init");
                        println!("或使用 --force 参数强制使用默认配置");
                        return Ok(());
                    }
                }
            }
        }
    }

    match cli.command {
        Commands::Service(cmd) => cmd.run(),
        Commands::Cmd(cmd) => cmd.run(),
        Commands::Init { reset } => run_init(reset, cli.force),
    }
}
