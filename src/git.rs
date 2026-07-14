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

pub fn branch_exists(
    repository: &Path,
    name: &str,
) -> Result<bool> {
    let reference = format!("refs/heads/{name}");

    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("show-ref")
        .arg("--verify")
        .arg("--quiet")
        .arg(&reference)
        .status()
        .with_context(|| {
            format!(
                "failed to run git show-ref in {}",
                repository.display()
            )
        })?;

    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => {
            bail!(
				"git show-ref failed in {}",
				repository.display()
			);
        }
    }
}

pub fn switch_work_branch(
    repository: &Path,
    name: &str,
    revision: &str,
    force: bool,
) -> Result<()> {
    let branch = format!("mws/{name}");

    if force {
        run(
            repository,
            &[
                "switch",
                "-C",
                &branch,
                revision,
            ],
        )
    } else {
        run(
            repository,
            &[
                "switch",
                "-c",
                &branch,
                revision,
            ],
        )
    }
}

pub fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

pub fn branches_with_prefix(
    repository: &Path,
    prefix: &str,
) -> Result<Vec<String>> {
    let pattern = format!("{prefix}*");
    let value = output(
        repository,
        &[
            "branch",
            "--list",
            &pattern,
            "--format=%(refname:short)",
        ],
    )?;

    Ok(value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

pub fn current_branch(repository: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("symbolic-ref")
        .arg("--quiet")
        .arg("--short")
        .arg("HEAD")
        .output()
        .with_context(|| {
            format!(
                "failed to run git symbolic-ref in {}",
                repository.display()
            )
        })?;

    if output.status.success() {
        let value = String::from_utf8(output.stdout)?;

        return Ok(Some(value.trim().to_owned()));
    }

    if output.status.code() == Some(1) {
        return Ok(None);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    bail!(
		"git symbolic-ref failed in {}: {}",
		repository.display(),
		stderr.trim()
	);
}

pub fn delete_branch(
    repository: &Path,
    branch: &str,
    force: bool,
) -> Result<()> {
    let option = if force {
        "-D"
    } else {
        "-d"
    };

    run(
        repository,
        &[
            "branch",
            option,
            branch,
        ],
    )
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
