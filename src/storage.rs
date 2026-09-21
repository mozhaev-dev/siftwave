use std::{
    io,
    path::{Path, PathBuf},
};
use tokio_rusqlite::rusqlite::{Connection, Error as SqliteError};

const DATABASE_FILE_NAME: &str = "app.sqlite";

pub fn database_path(workspace: &Path) -> PathBuf {
    workspace.join("data").join(DATABASE_FILE_NAME)
}

pub fn initialize(workspace: &Path) -> Result<(), SqliteError> {
    let connection = Connection::open(database_path(workspace))?;

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

pub fn validate(workspace: &Path) -> io::Result<()> {
    if !database_path(workspace).is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "SQLite file not found",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{database_path, initialize, validate};
    use std::{fs, io};
    use tokio_rusqlite::rusqlite::Connection;

    #[test]
    fn initialize_creates_database_and_schema_and_can_be_repeated() -> io::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(workspace.join("data"))?;

        initialize(&workspace).map_err(io::Error::other)?;
        initialize(&workspace).map_err(io::Error::other)?;

        let db_path = database_path(&workspace);
        assert!(db_path.is_file());

        let connection = Connection::open(db_path).map_err(io::Error::other)?;
        let table_count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema
                 WHERE type = 'table' AND name = 'topics'",
                [],
                |row| row.get(0),
            )
            .map_err(io::Error::other)?;

        assert_eq!(table_count, 1);

        Ok(())
    }

    #[test]
    fn validate_rejects_missing_database_file() -> io::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(workspace.join("data"))?;

        initialize(&workspace).map_err(io::Error::other)?;
        fs::remove_file(database_path(&workspace))?;

        let error =
            validate(&workspace).expect_err("validation should reject a missing SQLite file");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);

        Ok(())
    }
}
