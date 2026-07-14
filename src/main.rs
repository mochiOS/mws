mod cli;
mod commands;
mod hook;
mod manifest;
mod workspace;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init => commands::init::run()?,
        cli::Command::Restore { revision } => {
            commands::restore::run(&revision);
        }
        cli::Command::Status => commands::status::run(),
        cli::Command::Hook { command } => {
            commands::hook::run(command)?;
        }
    }

    Ok(())
}