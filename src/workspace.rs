use crate::storage;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const CURRENT_SCHEMA_VERSION: u32 = 1;
const CONFIG_NAME: &str = "siftwave.toml";
const DATABASE_FILE_NAME: &str = "app.sqlite";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WorkspaceConfig {
    schema_version: u32,
}

#[derive(Debug)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub data: PathBuf,
    pub episodes: PathBuf,
    pub config: PathBuf,
    pub database: PathBuf,
}

impl WorkspacePaths {
    pub fn new(path: &Path) -> Self {
        let data = path.join("data");

        Self {
            database: data.join(DATABASE_FILE_NAME),
            data,
            episodes: path.join("episodes"),
            config: path.join(CONFIG_NAME),
            root: PathBuf::from(path),
        }
    }

    pub fn create_dir_structure(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.data)?;
        fs::create_dir_all(&self.episodes)?;

        Ok(())
    }
}

pub fn initialize(path: &Path) -> io::Result<()> {
    let workspace_paths = WorkspacePaths::new(path);
    workspace_paths.create_dir_structure()?;

    storage::initialize(&workspace_paths.database).map_err(io::Error::other)?;

    if !workspace_paths.config.exists() {
        let config = WorkspaceConfig {
            schema_version: CURRENT_SCHEMA_VERSION,
        };

        let content = toml::to_string_pretty(&config).map_err(io::Error::other)?;
        fs::write(workspace_paths.config, content)?;
    }
    Ok(())
}

pub fn validate(path: &Path) -> io::Result<()> {
    let workspace_paths = WorkspacePaths::new(path);

    let config_content = fs::read_to_string(workspace_paths.config)?;

    let config: WorkspaceConfig = toml::from_str(&config_content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    if config.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported workspace schema version: {}",
                config.schema_version
            ),
        ));
    }

    storage::validate(&workspace_paths.database)?;

    Ok(())
}

#[cfg(test)]
mod tests;
