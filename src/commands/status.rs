use anyhow::{Context, Result};
use std::collections::BTreeMap;

use crate::git;
use crate::history;
use crate::snapshot;
use crate::workspace::Workspace;

pub fn run() -> Result<()> {
    let workspace = Workspace::discover()?;
    let snapshot_id = history::resolve_snapshot_id(&workspace, "latest")?;
    let snapshot = snapshot::load(&workspace, &snapshot_id)?;

    let mut work_branches: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut status_lines = Vec::new();

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

        if let Some(branch) = git::current_branch(&repository)? {
            if branch.starts_with("mws/") {
                work_branches
                    .entry(branch)
                    .or_default()
                    .push(project.path.display().to_string());
            }
        }

        let current_head = git::head(&repository)?;
        let dirty = git::is_dirty(&repository)?;
        let head_changed = current_head != project.head;

        if !dirty && !head_changed {
            continue;
        }

        if dirty && head_changed {
            status_lines.push(format!(
                "modified dirty {:<28} {} -> {}",
                project.path.display(),
                git::short_hash(&project.head),
                git::short_hash(&current_head)
            ));
        } else if dirty {
            status_lines.push(format!(
                "dirty    {:<28} {}",
                project.path.display(),
                git::short_hash(&current_head)
            ));
        } else {
            status_lines.push(format!(
                "modified {:<28} {} -> {}",
                project.path.display(),
                git::short_hash(&project.head),
                git::short_hash(&current_head)
            ));
        }
    }

    println!("snapshot {}", snapshot.id);

    print_work_branches(
        &work_branches,
        snapshot.projects.len(),
    );

    for _ in 0..32 { print!("-"); }

    if !work_branches.is_empty() || !status_lines.is_empty() {
        println!();
    }

    if status_lines.is_empty() {
        println!("clean");
    } else {
        for line in status_lines {
            println!("{}", line);
        }
    }

    Ok(())
}

fn print_work_branches(
    work_branches: &BTreeMap<String, Vec<String>>,
    project_count: usize,
) {
    if work_branches.is_empty() {
        return;
    }

    let work_project_count = work_branches
        .values()
        .map(Vec::len)
        .sum::<usize>();

    if work_branches.len() == 1 && work_project_count == project_count {
        let branch = work_branches
            .keys()
            .next()
            .expect("work branch should exist");

        println!("\x1b[32mwork {}\x1b[0m", display_work_name(branch));
        return;
    }

    println!("work branches:");

    for (branch, projects) in work_branches {
        for project in projects {
            println!(
                "  {:<28} {}",
                project,
                branch
            );
        }
    }
}

fn display_work_name(branch: &str) -> &str {
    branch.strip_prefix("mws/").unwrap_or(branch)
}