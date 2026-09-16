use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use tokio_rusqlite::rusqlite::{Connection, Error as SQLiteErr};

const CURRENT_SCHEMA_VERSION: u32 = 1;
const CONFIG_NAME: &str = "siftwave.toml";
const DB_NAME: &str = "app.sqlite";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WorkspaceConfig {
    schema_version: u32,
}

pub fn initialize(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("data"))?;
    fs::create_dir_all(path.join("episodes"))?;

    initialize_db(path).map_err(io::Error::other)?;

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

pub fn validate(path: &Path) -> io::Result<()> {
    let db_path = path.join("data").join(DB_NAME);
    if !db_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "SQLite file not found",
        ));
    }

    let config_content = fs::read_to_string(path.join(CONFIG_NAME))?;

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

    Ok(())
}

fn initialize_db(path: &Path) -> Result<(), SQLiteErr> {
    let db_path = path.join("data").join(DB_NAME);
    let connection = Connection::open(&db_path)?;

    let init_sql = "
        CREATE TABLE IF NOT EXISTS topics (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT ''
        ) STRICT;
    ";

    connection.execute_batch(init_sql)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_NAME, CURRENT_SCHEMA_VERSION, DB_NAME, initialize, validate};
    use std::{fs, io};
    use tokio_rusqlite::rusqlite::Connection;

    #[test]
    fn initialize_creates_directories_and_can_be_repeated() -> io::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("workspace");
        initialize(&workspace)?;
        initialize(&workspace)?;

        assert!(workspace.is_dir());
        assert!(workspace.join("data").is_dir());
        assert!(workspace.join("episodes").is_dir());

        let db_path = workspace.join("data").join(DB_NAME);
        assert!(db_path.is_file());

        let connection = Connection::open(db_path).map_err(io::Error::other)?;
        let sql = "
            SELECT count(*) FROM sqlite_schema
            WHERE type = 'table' AND name = 'topics';
        ";

        let res: i64 = connection
            .query_row(sql, [], |row| row.get(0))
            .map_err(io::Error::other)?;

        assert_eq!(res, 1);

        let config_content = fs::read_to_string(workspace.join(CONFIG_NAME))?;
        let config: super::WorkspaceConfig =
            toml::from_str(&config_content).map_err(io::Error::other)?;

        assert_eq!(
            config,
            super::WorkspaceConfig {
                schema_version: CURRENT_SCHEMA_VERSION
            }
        );

        validate(&workspace)?;

        Ok(())
    }

    #[test]
    fn validate_rejects_uninitialized_directory() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let err = validate(tmp.path()).expect_err("validation should fail without a config");

        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn validate_rejects_unsupported_schema_version() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let workspace = tmp.path().join("workspace");

        initialize(&workspace)?;

        fs::write(workspace.join(CONFIG_NAME), "schema_version = 999\n")?;

        let error = validate(&workspace)
            .expect_err("validation should reject an unsupported schema version");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        Ok(())
    }

    #[test]
    fn validate_rejects_no_sqlite_file() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let workspace = tmp.path().join("workspace");

        initialize(&workspace)?;
        fs::remove_file(workspace.join("data").join(DB_NAME))?;
        validate(&workspace).expect_err("validation should reject deleted SQLite file");

        Ok(())
    }
}
