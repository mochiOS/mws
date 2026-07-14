use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::snapshot;
use crate::workspace::Workspace;
use crate::{git, history};

pub fn run(
    snapshot_id: &str,
    force: bool,
    work: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let workspace = Workspace::discover()?;
    let resolved_snapshot_id = history::resolve_snapshot_id(
        &workspace,
        snapshot_id,
    )?;

    let snapshot = snapshot::load(
        &workspace,
        &resolved_snapshot_id,
    )?;

    if snapshot.projects.is_empty() {
        bail!("snapshot contains no projects: {}", snapshot.id);
    }

    if dry_run {
        return print_restore_plan(
            &workspace,
            &snapshot,
            force,
            work,
        );
    }

    let dirty_projects = find_dirty_projects(
        workspace.root(),
        &snapshot.projects,
    )?;

    if !dirty_projects.is_empty() && !force {
        for project in &dirty_projects {
            eprintln!("mws: dirty: {}", project);
        }

        bail!(
			"workspace has dirty repositories; use --force to discard changes"
		);
    }

    if let Some(work) = work {
        check_work_branches(
            workspace.root(),
            &snapshot.projects,
            work,
            force,
        )?;
    }

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

        if force && git::is_dirty(&repository)? {
            eprintln!(
                "mws: force clean: {}",
                project.path.display()
            );

            git::force_clean(&repository)?;
        }

        if let Some(work) = work {
            git::switch_work_branch(
                &repository,
                work,
                &project.head,
                force,
            )?;

            eprintln!(
                "mws: switch: {} -> mws/{} ({})",
                project.path.display(),
                work,
                project.head
            );
        } else {
            git::checkout(&repository, &project.head)?;

            eprintln!(
                "mws: checkout: {} -> {}",
                project.path.display(),
                project.head
            );
        }
    }

    eprintln!("mws: restored: {}", snapshot.id);

    Ok(())
}

fn find_dirty_projects(
    workspace_root: &Path,
    projects: &[snapshot::LoadedSnapshotProject],
) -> Result<Vec<String>> {
    let mut dirty_projects = Vec::new();

    for project in projects {
        let repository = workspace_root
            .join(&project.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "repository does not exist: {}",
                    workspace_root.join(&project.path).display()
                )
            })?;

        if git::is_dirty(&repository)? {
            dirty_projects.push(project.path.display().to_string());
        }
    }

    Ok(dirty_projects)
}

fn check_work_branches(
    workspace_root: &Path,
    projects: &[snapshot::LoadedSnapshotProject],
    work: &str,
    force: bool,
) -> Result<()> {
    if force {
        return Ok(());
    }

    let branch = format!("mws/{work}");
    let mut existing = Vec::new();

    for project in projects {
        let repository = workspace_root
            .join(&project.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "repository does not exist: {}",
                    workspace_root.join(&project.path).display()
                )
            })?;

        if git::branch_exists(&repository, &branch)? {
            existing.push(project.path.display().to_string());
        }
    }

    if !existing.is_empty() {
        for project in &existing {
            eprintln!(
                "mws: branch already exists: {} in {}",
                branch,
                project
            );
        }

        bail!(
			"work branch already exists; use --force to recreate it"
		);
    }

    Ok(())
}

fn print_restore_plan(
    workspace: &Workspace,
    snapshot: &snapshot::LoadedSnapshot,
    force: bool,
    work: Option<&str>,
) -> Result<()> {
    let mut dirty_projects = Vec::new();
    let mut branch_conflicts = Vec::new();

    println!("snapshot {}", snapshot.id);

    if let Some(work) = work {
        println!("\x1b[32mwork {}\x1b[0m", work);
    }

    println!();
    println!("restore plan:");

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

        if dirty {
            dirty_projects.push(project.path.display().to_string());
        }

        if let Some(work) = work {
            let branch = format!("mws/{work}");
            let exists = git::branch_exists(
                &repository,
                &branch,
            )?;

            if exists && !force {
                branch_conflicts.push(project.path.display().to_string());

                println!(
                    "  {:<28} would fail: {} already exists",
                    project.path.display(),
                    branch
                );
            } else if exists {
                println!(
                    "  {:<28} reset {} to {}",
                    project.path.display(),
                    branch,
                    git::short_hash(&project.head)
                );
            } else {
                println!(
                    "  {:<28} create {} at {}",
                    project.path.display(),
                    branch,
                    git::short_hash(&project.head)
                );
            }

            continue;
        }

        if current_head == project.head {
            if dirty {
                println!(
                    "  {:<28} unchanged ({}) dirty",
                    project.path.display(),
                    git::short_hash(&current_head)
                );
            } else {
                println!(
                    "  {:<28} unchanged ({})",
                    project.path.display(),
                    git::short_hash(&current_head)
                );
            }
        } else if dirty {
            println!(
                "  {:<28} {} -> {} dirty",
                project.path.display(),
                git::short_hash(&current_head),
                git::short_hash(&project.head)
            );
        } else {
            println!(
                "  {:<28} {} -> {}",
                project.path.display(),
                git::short_hash(&current_head),
                git::short_hash(&project.head)
            );
        }
    }

    if !dirty_projects.is_empty() {
        println!();
        println!("dirty repositories:");

        for project in &dirty_projects {
            println!("  {}", project);
        }

        if force {
            println!();
            println!("dirty changes would be discarded because --force is set");
        } else {
            println!();
            println!("restore would fail because --force is not set");
        }
    }

    if !branch_conflicts.is_empty() {
        println!();
        println!("work branch conflicts:");

        for project in &branch_conflicts {
            println!("  {}", project);
        }

        println!();
        println!("restore would fail because --force is not set");
    }

    println!();
    println!("no changes applied");

    Ok(())
}