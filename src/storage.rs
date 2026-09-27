use std::{io, path::Path};
use tokio_rusqlite::rusqlite::{Connection, Error as SqliteError};

use crate::{
    episode::{Episode, WorkflowStep},
    topic::Topic,
};

pub fn initialize(database_path: &Path) -> Result<(), SqliteError> {
    let connection = Connection::open(database_path)?;

    let init_sql = "
        CREATE TABLE IF NOT EXISTS topics (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT ''
        ) STRICT;

        CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY,
            topic_id INTEGER NOT NULL,
            topic_name TEXT NOT NULL,
            topic_description TEXT NOT NULL,
            current_step TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (topic_id) REFERENCES topics(id)
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

pub async fn create_episode(
    database: &tokio_rusqlite::Connection,
    topic_id: i64,
) -> Result<Episode, tokio_rusqlite::Error<SqliteError>> {
    let current_step = WorkflowStep::FindSources;

    database
        .call(move |connection| -> Result<Episode, SqliteError> {
            connection.query_row(
                "
                    INSERT INTO episodes (
                        topic_id,
                        topic_name,
                        topic_description,
                        current_step
                    )
                    SELECT
                        id, name, description, ?2
                    FROM
                        topics
                    WHERE
                        id = ?1
                    RETURNING
                        id,
                        topic_id,
                        topic_name,
                        topic_description,
                        version,
                        created_at
                ",
                tokio_rusqlite::rusqlite::params![topic_id, current_step.as_str()],
                |row| {
                    Ok(Episode {
                        id: row.get(0)?,
                        topic_id: row.get(1)?,
                        topic_name: row.get(2)?,
                        topic_description: row.get(3)?,
                        current_step,
                        version: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                },
            )
        })
        .await
}

#[cfg(test)]
mod tests;
