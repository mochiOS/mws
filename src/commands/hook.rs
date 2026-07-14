use anyhow::Result;

use crate::cli::HookCommand;

pub fn run(command: HookCommand) -> Result<()> {
    match command {
        HookCommand::PostCommit {
            workspace: _,
            repository: _,
        } => {
            run_post_commit()
        }
    }
}

fn run_post_commit() -> Result<()> {
    Ok(())
}