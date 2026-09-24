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

pub async fn list_topics(
    database: &tokio_rusqlite::Connection,
) -> Result<Vec<Topic>, tokio_rusqlite::Error<SqliteError>> {
    database
        .call(|connection| -> Result<Vec<Topic>, SqliteError> {
            let mut statement = connection.prepare(
                "
                    SELECT id, name, description
                    FROM topics
                    ORDER BY id
            ",
            )?;

            let topics = statement
                .query_map([], |row| {
                    Ok(Topic {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(topics)
        })
        .await
}

#[cfg(test)]
mod tests;
