mod cli;
mod commands;
pub mod workspace;
pub mod manifest;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init => commands::init::run()?,
        cli::Command::Restore { revision } => {
            commands::restore::run(&revision)
        }
        cli::Command::Status => commands::status::run(),
    }

    Ok(())
}