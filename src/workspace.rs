use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn discover() -> Result<Self> {
        let mut path = env::current_dir()?;

        loop {
            if path.join(".repo").is_dir() {
                return Ok(Self { root: path });
            }

            if !path.pop() {
                break;
            }
        }

        bail!("not inside a repo workspace")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn workspace_directory(&self) -> PathBuf {
        self.root.join(".workspace")
    }

    pub fn snapshot_directory(&self) -> PathBuf {
        self.workspace_directory().join("snapshots")
    }

    pub fn tree_path(&self) -> PathBuf {
        self.workspace_directory().join("tree.toml")
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into().canonicalize()?;

        if !root.join(".repo").is_dir() {
            bail!(
			"not a repo workspace: {}",
			root.display()
		);
        }

        Ok(Self { root })
    }
}