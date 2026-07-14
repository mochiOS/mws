use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::cli::HookCommand;
use crate::git;
use crate::manifest;
use crate::snapshot;
use crate::workspace::Workspace;

pub fn run(command: HookCommand) -> Result<()> {
    match command {
        HookCommand::PostCommit {
            workspace,
            repository,
        } => {
            run_post_commit(workspace, repository)
        }
    }
}

fn run_post_commit(
    workspace: Option<PathBuf>,
    repository: Option<PathBuf>,
) -> Result<()> {
    let workspace_path = workspace.context("missing --workspace")?;
    let repository_path = repository.context("missing --repository")?;

    let workspace = Workspace::from_root(workspace_path)?;
    let repository = repository_path.canonicalize()?;

    if !repository.starts_with(workspace.root()) {
        bail!(
			"repository escapes workspace: {}",
			repository.display()
		);
    }

    let projects = manifest::parse(&workspace)?;

    if projects.is_empty() {
        bail!("manifest contains no projects");
    }

    let trigger = find_trigger_project(&projects, workspace.root(), &repository)?;

    eprintln!("mws: post-commit: {}", trigger);
    eprintln!("mws: workspace: {}", workspace.root().display());

    for project in &projects {
        let project_repository = workspace
            .root()
            .join(&project.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "repository does not exist: {}",
                    workspace.root().join(&project.path).display()
                )
            })?;

        let head = git::head(&project_repository)?;
        let dirty = git::is_dirty(&project_repository)?;

        if dirty {
            eprintln!(
                "mws: {:<28} {} dirty",
                project.path.display(),
                head
            );
        } else {
            eprintln!(
                "mws: {:<28} {}",
                project.path.display(),
                head
            );
        }
    }

    let dirty_projects = find_dirty_projects(&projects, workspace.root())?;

    if !dirty_projects.is_empty() {
        eprintln!("mws: snapshot skipped: workspace has dirty repositories");

        for project in dirty_projects {
            eprintln!("mws: dirty: {}", project);
        }

        return Ok(());
    }

    let path = snapshot::save_current(
        &workspace,
        &projects,
        &trigger,
        &repository,
    )?;

    eprintln!("mws: saved: {}", path.display());

    Ok(())
}

fn find_trigger_project(
    projects: &[manifest::Project],
    workspace_root: &Path,
    repository: &Path,
) -> Result<String> {
    for project in projects {
        let project_repository = workspace_root
            .join(&project.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "repository does not exist: {}",
                    workspace_root.join(&project.path).display()
                )
            })?;

        if project_repository == repository {
            return Ok(project.name.clone());
        }
    }

    Ok(repository.display().to_string())
}

fn find_dirty_projects(
    projects: &[manifest::Project],
    workspace_root: &Path,
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