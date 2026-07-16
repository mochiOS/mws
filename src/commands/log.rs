use std::env;
use std::io::{self, IsTerminal};
use std::process::{Child, Command, Stdio};
use anyhow::{Context, Result};
use crate::history;
use crate::workspace::Workspace;

pub fn run(max_count: Option<usize>, all: bool) -> Result<()> {
    let workspace = Workspace::discover()?;

    let limit = if all {
        None
    } else {
        Some(max_count.unwrap_or(20))
    };

    if !io::stdout().is_terminal() {
        return print_direct(&workspace, limit);
    }

    let Some(mut pager) = spawn_pager()? else {
        return print_direct(&workspace, limit);
    };

    {
        let mut stdin = pager
            .stdin
            .take()
            .context("pager stdin is unavailable")?;

        if let Err(error) = history::print(&workspace, limit, &mut stdin) {
            let broken_pipe = error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| {
                    error.kind() == io::ErrorKind::BrokenPipe
                });

            if !broken_pipe {
                return Err(error);
            }
        }
    }

    pager.wait()?;

    Ok(())
}

fn configured_pager() -> Option<String> {
    env::var("MWS_PAGER")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("PAGER")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn spawn_pager() -> Result<Option<Child>> {
    let mut command;

    if let Some(pager) = configured_pager() {
        command = Command::new("sh");
        command.arg("-c").arg(pager);
    } else {
        command = Command::new("less");
    }

    command.stdin(Stdio::piped());

    if env::var_os("LESS").is_none() {
        command.env("LESS", "FRX");
    }

    match command.spawn() {
        Ok(child) => Ok(Some(child)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("failed to start pager"),
    }
}

fn print_direct(
    workspace: &Workspace,
    limit: Option<usize>,
) -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    history::print(workspace, limit, &mut stdout)
}