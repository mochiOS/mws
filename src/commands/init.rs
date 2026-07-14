use anyhow::Result;

use crate::manifest;
use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;
    let projects = manifest::parse(&workspace)?;

    println!("Workspace: {}", workspace.root().display());
    println!();

    for project in projects {
        println!(
            "{} -> {}",
            project.name,
            project.path.display()
        );
    }

    Ok(())
}