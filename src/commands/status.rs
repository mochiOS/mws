use anyhow::{Context, Result};

use crate::git;
use crate::history;
use crate::snapshot;
use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;
    let snapshot_id = history::resolve_snapshot_id(&workspace, "latest")?;
    let snapshot = snapshot::load(&workspace, &snapshot_id)?;

    let mut changed = false;

    println!("snapshot {}", snapshot.id);

    for project in &snapshot.projects {
        let repository = workspace
            .root()
            .join(&project.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "repository does not exist: {}",
                    workspace.root().join(&project.path).display()
                )
            })?;

        let current_head = git::head(&repository)?;
        let dirty = git::is_dirty(&repository)?;
        let head_changed = current_head != project.head;

        if !dirty && !head_changed {
            continue;
        }

        changed = true;

        if dirty && head_changed {
            println!(
                "modified dirty {:<28} {} -> {}",
                project.path.display(),
                git::short_hash(&project.head),
                git::short_hash(&current_head)
            );
        } else if dirty {
            println!(
                "dirty    {:<28} {}",
                project.path.display(),
                git::short_hash(&current_head)
            );
        } else {
            println!(
                "modified {:<28} {} -> {}",
                project.path.display(),
                git::short_hash(&project.head),
                git::short_hash(&current_head)
            );
        }
    }

    if !changed {
        println!("clean");
    }

    Ok(())
}