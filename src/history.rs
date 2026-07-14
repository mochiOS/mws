use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::workspace::Workspace;

const TREE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Tree {
    version: u32,
    entries: Vec<TreeEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TreeEntry {
    date: String,
    snapshot: String,
    path: String,
    hash: String,
    author: String,
    message: String,
}

pub struct AppendEntry {
    pub date: String,
    pub snapshot: String,
    pub path: String,
    pub message: String,
    pub hash: String,
    pub author: String,
}

pub fn append(
    workspace: &Workspace,
    entry: AppendEntry,
) -> Result<()> {
    let path = workspace.tree_path();
    let mut tree = load_tree(&path)?;

    if !tree.entries.iter().any(|item| item.snapshot == entry.snapshot) {
        tree.entries.push(TreeEntry {
            date: entry.date,
            snapshot: entry.snapshot,
            path: entry.path,
            message: entry.message,
            hash: entry.hash,
            author: entry.author,
        });
    }

    save_tree(&path, &tree)?;

    Ok(())
}

pub fn print(workspace: &Workspace) -> Result<()> {
    let tree = load_tree(&workspace.tree_path())?;

    for entry in tree.entries.iter().rev() {
        println!("\x1b[33mcommit {}\x1b[0m", entry.hash);
        println!("Author: {}", entry.author);
        println!("Date:   {}", format_date(&entry.date));
        println!();
        println!("\t{}: {}", entry.path, entry.message);
        println!();
    }

    Ok(())
}

pub fn resolve_snapshot_id(
    workspace: &Workspace,
    target: &str,
) -> Result<String> {
    let tree = load_tree(&workspace.tree_path())?;

    if target == "latest" {
        let Some(entry) = tree.entries.last() else {
            bail!("workspace history is empty");
        };

        return Ok(entry.snapshot.clone());
    }

    let mut matches = Vec::new();

    for entry in &tree.entries {
        if entry.snapshot == target
            || entry.snapshot.starts_with(target)
            || entry.hash == target
            || entry.hash.starts_with(target)
        {
            matches.push(entry.snapshot.clone());
        }
    }

    matches.sort();
    matches.dedup();

    match matches.len() {
        0 => {
            bail!("snapshot or commit not found: {}", target)
        }
        1 => {
            Ok(matches.remove(0))
        }
        _ => {
            bail!("ambiguous snapshot or commit: {}", target)
        }
    }
}

fn load_tree(path: &Path) -> Result<Tree> {
    if !path.exists() {
        return Ok(Tree {
            version: TREE_VERSION,
            entries: Vec::new(),
        });
    }

    let content = fs::read_to_string(path)?;
    let tree = toml::from_str(&content)?;

    Ok(tree)
}

fn save_tree(path: &Path, tree: &Tree) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension("toml.tmp");
    let content = toml::to_string_pretty(tree)?;

    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

fn format_date(value: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(date) => date.format("%Y-%m-%d %H:%M:%S").to_string(),
        Err(_) => value.to_owned(),
    }
}