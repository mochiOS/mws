use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::manifest::Project;
use crate::workspace::Workspace;

const MWS_HOOK_MARKER: &str = "# mws-managed-hook";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookAction {
    Install,
    Update,
}

impl HookAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Install => "installed",
            Self::Update => "updated",
        }
    }
}

pub struct HookPlan {
    pub project_name: String,
    pub repository: PathBuf,
    pub hook_path: PathBuf,
    pub action: HookAction,
}

pub fn plan_post_commit(
    workspace: &Workspace,
    project: &Project,
) -> Result<HookPlan> {
    let workspace_root = workspace.root().canonicalize()?;
    let repository_path = workspace.root().join(&project.path);

    let repository = repository_path.canonicalize().with_context(|| {
        format!(
            "repository does not exist: {}",
            repository_path.display()
        )
    })?;

    if !repository.starts_with(&workspace_root) {
        bail!(
			"project path escapes workspace: {}",
			project.path.display()
		);
    }

    ensure_git_repository(&repository)?;

    let hook_path = git_hook_path(&repository, "post-commit")?;
    let action = if hook_path.exists() {
        ensure_hook_is_safe_to_update(&hook_path)?;
        HookAction::Update
    } else {
        HookAction::Install
    };

    Ok(HookPlan {
        project_name: project.name.clone(),
        repository,
        hook_path,
        action,
    })
}

pub fn install_post_commit(
    workspace: &Workspace,
    plan: &HookPlan,
) -> Result<()> {
    let workspace_root = workspace.root().canonicalize()?;
    let executable = std::env::current_exe()?.canonicalize()?;

    if let Some(parent) = plan.hook_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let script = format!(
        "#!/bin/sh\n{}\nexec {} hook post-commit --workspace {} --repository {}\n",
        MWS_HOOK_MARKER,
        shell_quote(&executable),
        shell_quote(&workspace_root),
        shell_quote(&plan.repository),
    );

    let file_name = plan
        .hook_path
        .file_name()
        .context("hook path has no file name")?;

    let temp_path = plan
        .hook_path
        .with_file_name(format!("{}.mws-tmp", file_name.to_string_lossy()));

    fs::write(&temp_path, script)?;
    set_executable(&temp_path)?;

    fs::rename(&temp_path, &plan.hook_path).with_context(|| {
        format!(
            "failed to install hook: {}",
            plan.hook_path.display()
        )
    })?;

    Ok(())
}

fn ensure_git_repository(repository: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .with_context(|| {
            format!(
                "failed to run git in {}",
                repository.display()
            )
        })?;

    if !output.status.success() {
        bail!(
			"not a git repository: {}",
			repository.display()
		);
    }

    let value = String::from_utf8(output.stdout)?;

    if value.trim() != "true" {
        bail!(
			"not inside git work tree: {}",
			repository.display()
		);
    }

    Ok(())
}

fn ensure_hook_is_safe_to_update(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;

    if metadata.file_type().is_symlink() {
        bail!(
			"refusing to overwrite symlink hook: {}",
			path.display()
		);
    }

    let content = fs::read(path)?;

    if !contains_marker(&content) {
        bail!(
			"post-commit hook already exists and is not managed by mws: {}",
			path.display()
		);
    }

    Ok(())
}

fn contains_marker(content: &[u8]) -> bool {
    let marker = MWS_HOOK_MARKER.as_bytes();

    content
        .windows(marker.len())
        .any(|window| window == marker)
}

fn git_hook_path(repository: &Path, name: &str) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("rev-parse")
        .arg("--git-path")
        .arg(format!("hooks/{name}"))
        .output()
        .with_context(|| {
            format!(
                "failed to run git in {}",
                repository.display()
            )
        })?;

    if !output.status.success() {
        bail!(
			"failed to resolve git hook path: {}",
			repository.display()
		);
    }

    let value = String::from_utf8(output.stdout)?;
    let path = PathBuf::from(value.trim());

    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(repository.join(path))
    }
}

fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;

    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}