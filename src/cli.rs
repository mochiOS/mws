use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mws",
    author,
    version,
    about = "Workspace manager utilizing repo"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize workspace
    Init,

    /// Restore workspace
    Restore {
        revision: String,
    },

    /// Show workspace status
    Status,

    /// Internal hook entrypoint
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Called from git post-commit hook
    PostCommit {
        #[arg(long)]
        workspace: Option<PathBuf>,

        #[arg(long)]
        repository: Option<PathBuf>,
    },
}