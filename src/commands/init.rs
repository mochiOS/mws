use anyhow::Result;

use crate::hook;
use crate::manifest;
use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;
    let projects = manifest::parse(&workspace)?;

    println!("Workspace: {}", workspace.root().display());
    for _ in 0..31 {
        print!("-");
    }
    println!();

    for project in &projects {
        let plan = hook::plan_post_commit(&workspace, project)?;
        hook::install_post_commit(&workspace, &plan)?;

        println!(
            "installed hook: {} -> {}",
            project.name,
            project.path.display()
        );
    }

    println!();
    println!("{} repositories initialized", projects.len());

    Ok(())
}