use crate::git;
use crate::manifest::Project;
use crate::workspace::Workspace;
use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const SNAPSHOT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    id: String,
    created: String,
    trigger: SnapshotTrigger,
    projects: Vec<SnapshotProject>,
}

#[derive(Serialize, Deserialize)]
struct SnapshotTrigger {
    name: String,
    path: String,
    repository: String,
    head: String,
    author: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct SnapshotProject {
    name: String,
    path: String,
    head: String,
}

pub struct SavedSnapshot {
    pub id: String,
    pub path: PathBuf,
    pub created: String,
    pub trigger_path: String,
    pub trigger_head: String,
    pub trigger_author: String,
    pub trigger_message: String,
}

pub struct LoadedSnapshot {
    pub id: String,
    pub projects: Vec<LoadedSnapshotProject>,
}

#[allow(unused)]
pub struct LoadedSnapshotProject {
    pub name: String,
    pub path: PathBuf,
    pub head: String,
}

pub fn load(
    workspace: &Workspace,
    id: &str,
) -> Result<LoadedSnapshot> {
    let file_name = if id.ends_with(".toml") {
        id.to_owned()
    } else {
        format!("{id}.toml")
    };

    let path = workspace.snapshot_directory().join(file_name);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read snapshot: {}",
            path.display()
        )
    })?;

    let snapshot: Snapshot = toml::from_str(&content)?;

    Ok(LoadedSnapshot {
        id: snapshot.id,
        projects: snapshot
            .projects
            .into_iter()
            .map(|project| LoadedSnapshotProject {
                name: project.name,
                path: PathBuf::from(project.path),
                head: project.head,
            })
            .collect(),
    })
}

pub fn save_current(
    workspace: &Workspace,
    projects: &[Project],
    trigger: &Project,
    trigger_repository: &Path,
) -> Result<SavedSnapshot> {
    if projects.is_empty() {
        bail!("refusing to save empty snapshot");
    }

    let created = Local::now().to_rfc3339();

    let snapshot = collect_snapshot(
        workspace,
        projects,
        trigger,
        trigger_repository,
        created.clone(),
    )?;

    if snapshot.projects.is_empty() {
        bail!("refusing to save snapshot with no projects");
    }

    let directory = workspace.snapshot_directory();

    fs::create_dir_all(&directory)?;

    let path = directory.join(format!("{}.toml", snapshot.id));
    let temp_path = path.with_extension("toml.tmp");

    let content = toml::to_string_pretty(&snapshot)?;

    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, &path)?;

    Ok(SavedSnapshot {
        id: snapshot.id.clone(),
        path,
        created,
        trigger_path: trigger.path.display().to_string(),
        trigger_head: snapshot.trigger.head.clone(),
        trigger_author: snapshot.trigger.author.clone(),
        trigger_message: snapshot.trigger.message.clone(),
    })
}

fn collect_snapshot(
    workspace: &Workspace,
    projects: &[Project],
    trigger: &Project,
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

    let trigger_head = git::head(trigger_repository)?;
    let trigger_author = git::commit_author(
        trigger_repository,
        &trigger_head,
    )?;
    let trigger_message = git::commit_subject(
        trigger_repository,
        &trigger_head,
    )?;
    let id = snapshot_id(&snapshot_projects);

    Ok(Snapshot {
        version: SNAPSHOT_VERSION,
        id,
        created,
        trigger: SnapshotTrigger {
            name: trigger.name.clone(),
            path: trigger.path.display().to_string(),
            repository: trigger_repository.display().to_string(),
            head: trigger_head,
            author: trigger_author,
            message: trigger_message,
        },
        projects: snapshot_projects,
    })
}

fn snapshot_id(projects: &[SnapshotProject]) -> String {
    let mut hasher = Sha256::new();

    for project in projects {
        hasher.update(project.name.as_bytes());
        hasher.update(b"\0");
        hasher.update(project.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(project.head.as_bytes());
        hasher.update(b"\0");
    }

    let digest = hasher.finalize();

    digest
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}