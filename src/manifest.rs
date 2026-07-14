use std::path::PathBuf;

pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

// TODO: repo manifest XMLの解析