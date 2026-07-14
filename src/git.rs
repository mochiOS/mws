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

pub fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}