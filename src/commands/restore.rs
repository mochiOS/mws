use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::snapshot;
use crate::workspace::Workspace;
use crate::{git, history};

pub fn run(
    snapshot_id: &str,
    force: bool,
    work: Option<&str>,
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