use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::cli::WorkCommand;
use crate::git;
use crate::manifest;
use crate::workspace::Workspace;

struct CleanPlan {
    project_path: String,
    repository: PathBuf,
    branch: String,
}

pub fn run(command: WorkCommand) -> Result<()> {
    match command {
        WorkCommand::Clean {
            branch,
            force,
        } => clean(&branch, force),
    }
}

fn clean(
    branch: &str,
    force: bool,
) -> Result<()> {
    let workspace = Workspace::discover()?;
    let projects = manifest::parse(&workspace)?;
    let branch = normalize_work_branch(branch)?;

    let mut plans = Vec::new();
    let mut checked_out = Vec::new();

    for project in &projects {
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

        if !git::branch_exists(&repository, &branch)? {
            continue;
        }

        if let Some(current) = git::current_branch(&repository)? {
            if current == branch {
                checked_out.push(format!(
                    "{}: {}",
                    project.path.display(),
                    current
                ));
            }
        }

        plans.push(CleanPlan {
            project_path: project.path.display().to_string(),
            repository,
            branch: branch.clone(),
        });
    }

    if !checked_out.is_empty() {
        for item in &checked_out {
            eprintln!("mws: checked out branch: {}", item);
        }

        bail!("cannot delete checked out branch; switch branches first");
    }

    if plans.is_empty() {
        println!("mws: branch not found: {}", branch);
        return Ok(());
    }

    println!("mws: delete branch {}", branch);
    println!();

    for plan in &plans {
        println!("  {}", plan.project_path);
    }

    println!();
    print!("continue? [y/N] ");
    io::stdout().flush()?;

    if !confirm()? {
        println!("mws: cancelled");
        return Ok(());
    }

    for plan in &plans {
        git::delete_branch(
            &plan.repository,
            &plan.branch,
            force,
        )?;

        println!(
            "deleted {}: {}",
            plan.project_path,
            plan.branch
        );
    }

    Ok(())
}

fn normalize_work_branch(branch: &str) -> Result<String> {
    let branch = branch.trim();

    if branch.is_empty() {
        bail!("branch name is empty");
    }

    if branch == "mws/" {
        bail!("branch name is empty");
    }

    if branch.chars().any(char::is_whitespace) {
        bail!("branch name contains whitespace: {}", branch);
    }

    if let Some(name) = branch.strip_prefix("mws/") {
        if name.is_empty() {
            bail!("branch name is empty");
        }

        return Ok(format!("mws/{name}"));
    }

    Ok(format!("mws/{branch}"))
}

fn confirm() -> Result<bool> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();

    Ok(input == "y" || input == "Y" || input == "yes")
}