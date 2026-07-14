use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn head(repository: &Path) -> Result<String> {
    let value = output(repository, &["rev-parse", "HEAD"])?;

    Ok(value.trim().to_owned())
}

pub fn is_dirty(repository: &Path) -> Result<bool> {
    let value = output(
        repository,
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
        ],
    )?;

    Ok(!value.trim().is_empty())
}

fn output(repository: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to run git in {}",
                repository.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        bail!(
			"git command failed in {}: {}",
			repository.display(),
			stderr.trim()
		);
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn commit_subject(repository: &Path, revision: &str) -> Result<String> {
    let value = output(
        repository,
        &[
            "log",
            "-1",
            "--format=%s",
            revision,
        ],
    )?;

    Ok(value.trim_end().to_owned())
}

pub fn commit_paths(
    repository: &Path,
    paths: &[&Path],
    message: &str,
) -> Result<bool> {
    for path in paths {
        let relative_path = path.strip_prefix(repository).with_context(|| {
            format!(
                "path is outside repository: {}",
                path.display()
            )
        })?;

        let output = Command::new("git")
            .arg("-C")
            .arg(repository)
            .arg("add")
            .arg("--")
            .arg(relative_path)
            .output()
            .with_context(|| {
                format!(
                    "failed to run git add in {}",
                    repository.display()
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            bail!(
				"git add failed in {}: {}",
				repository.display(),
				stderr.trim()
			);
        }
    }

    if !has_staged_changes(repository)? {
        return Ok(false);
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .env("MWS_INTERNAL_COMMIT", "1")
        .status()
        .with_context(|| {
            format!(
                "failed to run git commit in {}",
                repository.display()
            )
        })?;

    if !status.success() {
        bail!(
			"git commit failed in {}",
			repository.display()
		);
    }

    Ok(true)
}

fn has_staged_changes(repository: &Path) -> Result<bool> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .status()
        .with_context(|| {
            format!(
                "failed to run git diff in {}",
                repository.display()
            )
        })?;

    Ok(!status.success())
}

pub fn commit_author(repository: &Path, revision: &str) -> Result<String> {
    let value = output(
        repository,
        &[
            "log",
            "-1",
            "--format=%an <%ae>",
            revision,
        ],
    )?;

    Ok(value.trim_end().to_owned())
}

pub fn checkout(
    repository: &Path,
    revision: &str,
) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("checkout")
        .arg("--detach")
        .arg(revision)
        .output()
        .with_context(|| {
            format!(
                "failed to run git checkout in {}",
                repository.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        bail!(
			"git checkout failed in {}: {}",
			repository.display(),
			stderr.trim()
		);
    }

    Ok(())
}

pub fn force_clean(repository: &Path) -> Result<()> {
    run(repository, &["reset", "--hard"])?;
    run(repository, &["clean", "-fd"])?;

    Ok(())
}

fn run(repository: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to run git in {}",
                repository.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        bail!(
			"git command failed in {}: {}",
			repository.display(),
			stderr.trim()
		);
    }

    Ok(())
}