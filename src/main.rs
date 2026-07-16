mod cli;
mod commands;
mod hook;
mod manifest;
mod workspace;
mod git;
mod snapshot;
mod history;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init => commands::init::run()?,
        cli::Command::Restore {
            revision,
            force,
            work,
            dry_run,
        } => {
            commands::restore::run(&revision, force, work.as_deref(), dry_run)?;
        }
        cli::Command::Status => commands::status::run()?,
        cli::Command::Hook { command } => {
            commands::hook::run(command)?;
        },
        cli::Command::Log { max_count, all } => {
            commands::log::run(max_count, all)?;
        }
        cli::Command::Work { command } => commands::work::run(command)?,
    }

    Ok(())
}