use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::Serialize;

use crate::git;
use crate::manifest::Project;
use crate::workspace::Workspace;

const SNAPSHOT_VERSION: u32 = 1;

#[derive(Serialize)]
struct Snapshot {
    version: u32,
    created: String,
    trigger: SnapshotTrigger,
    projects: Vec<SnapshotProject>,
}

#[derive(Serialize)]
struct SnapshotTrigger {
    name: String,
    repository: String,
}

#[derive(Serialize)]
struct SnapshotProject {
    name: String,
    path: String,
    head: String,
}

pub fn save_current(
    workspace: &Workspace,
    projects: &[Project],
    trigger_name: &str,
    trigger_repository: &Path,
) -> Result<PathBuf> {
    if projects.is_empty() {
        bail!("refusing to save empty snapshot");
    }

    let created = Local::now();

    let snapshot = collect_snapshot(
        workspace,
        projects,
        trigger_name,
        trigger_repository,
        created.to_rfc3339(),
    )?;

    if snapshot.projects.is_empty() {
        bail!("refusing to save snapshot with no projects");
    }

    let directory = workspace.snapshot_directory();

    fs::create_dir_all(&directory)?;

    let file_name = format!(
        "{}-{}.toml",
        created.format("%Y%m%d-%H%M%S"),
        sanitize_file_name(trigger_name)
    );

    let path = directory.join(file_name);
    let temp_path = path.with_extension("toml.tmp");

    let content = toml::to_string_pretty(&snapshot)?;

    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, &path)?;

    Ok(path)
}

fn collect_snapshot(
    workspace: &Workspace,
    projects: &[Project],
    trigger_name: &str,
    trigger_repository: &Path,
    created: String,
) -> Result<Snapshot> {
    let mut snapshot_projects = Vec::new();

    for project in projects {
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

        let head = git::head(&repository)?;
        let dirty = git::is_dirty(&repository)?;

        if dirty {
            bail!(
				"repository has uncommitted changes: {}",
				project.path.display()
			);
        }

        snapshot_projects.push(SnapshotProject {
            name: project.name.clone(),
            path: project.path.display().to_string(),
            head,
        });
    }

    Ok(Snapshot {
        version: SNAPSHOT_VERSION,
        created,
        trigger: SnapshotTrigger {
            name: trigger_name.to_owned(),
            repository: trigger_repository.display().to_string(),
        },
        projects: snapshot_projects,
    })
}

fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.'
            {
                character
            } else {
                '_'
            }
        })
        .collect()
}