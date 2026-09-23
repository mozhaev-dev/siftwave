use std::{io, path::Path};
use tokio_rusqlite::rusqlite::{Connection, Error as SqliteError};

use crate::topic::Topic;

pub fn initialize(database_path: &Path) -> Result<(), SqliteError> {
    let connection = Connection::open(database_path)?;

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

pub fn validate(database_path: &Path) -> io::Result<()> {
    if !database_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "SQLite file not found",
        ));
    }

    Ok(())
}

pub async fn open(database_path: &Path) -> Result<tokio_rusqlite::Connection, SqliteError> {
    tokio_rusqlite::Connection::open(database_path).await
}

pub async fn create_topic(
    database: &tokio_rusqlite::Connection,
    name: String,
    description: String,
) -> Result<Topic, tokio_rusqlite::Error<SqliteError>> {
    database
        .call(move |connection| -> Result<Topic, SqliteError> {
            connection.execute(
                "
                INSERT INTO topics (name, description)
                VALUES (?1, ?2)
            ",
                tokio_rusqlite::rusqlite::params![&name, &description],
            )?;

            Ok(Topic {
                id: connection.last_insert_rowid(),
                name,
                description,
            })
        })
        .await
}

pub async fn get_topic_by_id(
    database: &tokio_rusqlite::Connection,
    id: i64,
) -> Result<Topic, tokio_rusqlite::Error<SqliteError>> {
    database
        .call(move |connection| -> Result<Topic, SqliteError> {
            connection.query_row(
                "
                SELECT id, name, description
                FROM topics
                WHERE id = ?1
                ",
                tokio_rusqlite::rusqlite::params![id],
                |row| {
                    Ok(Topic {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                    })
                },
            )
        })
        .await
}

#[cfg(test)]
mod tests {
    use crate::storage::{create_topic, get_topic_by_id, open};

    use super::{initialize, validate};
    use std::{fs, io};

    use tokio_rusqlite::rusqlite::Connection;
    #[test]
    fn initialize_creates_database_and_schema_and_can_be_repeated() -> io::Result<()> {
        let temp = tempfile::tempdir()?;
        let database_path = temp.path().join("app.sqlite");

        initialize(&database_path).map_err(io::Error::other)?;
        initialize(&database_path).map_err(io::Error::other)?;

        assert!(database_path.is_file());

        let connection = Connection::open(database_path).map_err(io::Error::other)?;
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
        let database_path = temp.path().join("app.sqlite");

        initialize(&database_path).map_err(io::Error::other)?;
        fs::remove_file(&database_path)?;

        let error =
            validate(&database_path).expect_err("validation should reject a missing SQLite file");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);

        Ok(())
    }

    #[tokio::test]
    async fn create_topic_inserts_topic() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let database_path = temp.path().join("app.sqlite");

        initialize(&database_path).map_err(io::Error::other)?;
        let database = open(&database_path).await?;

        let created = create_topic(
            &database,
            String::from("Rust Weekly"),
            String::from("Weekly Rust updates"),
        )
        .await?;

        let stored = get_topic_by_id(&database, created.id).await?;

        assert_eq!(stored, created);

        Ok(())
    }
}
