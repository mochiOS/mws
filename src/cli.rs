use clap::{Parser, Subcommand};

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
}