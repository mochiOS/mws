use anyhow::Result;

use crate::history;
use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;

    history::print(&workspace)?;

    Ok(())
}