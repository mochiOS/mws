use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::snapshot;
use crate::workspace::Workspace;
use crate::{git, history};

pub fn run(
    snapshot_id: &str,
    force: bool,
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

        git::checkout(&repository, &project.head)?;

        eprintln!(
            "mws: checkout: {} -> {}",
            project.path.display(),
            project.head
        );
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