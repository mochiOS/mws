use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{bail, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::workspace::Workspace;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub remote: Option<String>,
    pub revision: Option<String>,
}

pub fn parse(workspace: &Workspace) -> Result<Vec<Project>> {
    let mut projects = Vec::new();
    let mut visited = HashSet::new();

    parse_file(
        workspace.root().join(".repo/manifest.xml"),
        workspace,
        &mut visited,
        &mut projects,
    )?;

    Ok(projects)
}

fn parse_project(
    element: &BytesStart<'_>,
    projects: &mut Vec<Project>,
) -> Result<()> {
    let mut name = None;
    let mut path = None;
    let mut remote = None;
    let mut revision = None;

    for attribute in element.attributes() {
        let attribute = attribute?;

        let value = std::str::from_utf8(attribute.value.as_ref())?;

        match attribute.key.as_ref() {
            b"name" => {
                name = Some(value.to_owned());
            }

            b"path" => {
                path = Some(PathBuf::from(value));
            }

            b"remote" => {
                remote = Some(value.to_owned());
            }

            b"revision" => {
                revision = Some(value.to_owned());
            }

            _ => {}
        }
    }

    let Some(name) = name else {
        bail!("<project> missing name attribute");
    };

    projects.push(Project {
        path: path.unwrap_or_else(|| PathBuf::from(&name)),
        name,
        remote,
        revision,
    });

    Ok(())
}

fn parse_file(
    path: PathBuf,
    workspace: &Workspace,
    visited: &mut HashSet<PathBuf>,
    projects: &mut Vec<Project>,
) -> Result<()> {
    if !visited.insert(path.clone()) {
        return Ok(());
    }

    let file = File::open(&path)?;
    let mut reader = Reader::from_reader(BufReader::new(file));

    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Empty(element) | Event::Start(element) => {
                match element.name().as_ref() {
                    b"project" => {
                        parse_project(&element, projects)?;
                    }

                    b"include" => {
                        parse_include(
                            &element,
                            workspace,
                            visited,
                            projects,
                        )?;
                    }

                    _ => {}
                }
            }

            Event::Eof => break,

            _ => {}
        }

        buffer.clear();
    }

    Ok(())
}

fn parse_include(
    element: &BytesStart<'_>,
    workspace: &Workspace,
    visited: &mut HashSet<PathBuf>,
    projects: &mut Vec<Project>,
) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;

        if attribute.key.as_ref() != b"name" {
            continue;
        }

        let name = std::str::from_utf8(attribute.value.as_ref())?;

        let path = workspace
            .root()
            .join(".repo/manifests")
            .join(name);

        parse_file(path, workspace, visited, projects)?;

        return Ok(());
    }

    bail!("<include> missing name attribute")
}