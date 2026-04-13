use crate::commands::{CmdCmd, ServiceCmd};
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
