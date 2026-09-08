use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

const CURRENT_SCHEMA_VERSION: u32 = 1;
const CONFIG_NAME: &str = "siftwave.toml";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WorkspaceConfig {
    schema_version: u32,
}

pub fn initialize(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("data"))?;
    fs::create_dir_all(path.join("episodes"))?;

    let config_path = path.join(CONFIG_NAME);

    if !config_path.exists() {
        let config = WorkspaceConfig {
            schema_version: CURRENT_SCHEMA_VERSION,
        };

        let content = toml::to_string_pretty(&config).map_err(io::Error::other)?;
        fs::write(config_path, content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::workspace::{CONFIG_NAME, CURRENT_SCHEMA_VERSION};

    #[test]
    fn initialize_creates_directories_and_can_be_repeated() -> std::io::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("workspace");
        super::initialize(&workspace)?;
        super::initialize(&workspace)?;

        assert!(workspace.is_dir());
        assert!(workspace.join("data").is_dir());
        assert!(workspace.join("episodes").is_dir());

        let config_content = std::fs::read_to_string(workspace.join(CONFIG_NAME))?;
        let config: super::WorkspaceConfig =
            toml::from_str(&config_content).map_err(std::io::Error::other)?;

        assert_eq!(
            config,
            super::WorkspaceConfig {
                schema_version: CURRENT_SCHEMA_VERSION
            }
        );

        Ok(())
    }
}
