use anyhow::Result;

use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;

    println!("Workspace: {}", workspace.root().display());
    println!("Manifest : {}", workspace.manifest_path().display());

    Ok(())
}