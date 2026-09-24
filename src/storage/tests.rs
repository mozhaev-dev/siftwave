use super::{create_topic, get_topic_by_id, initialize, list_topics, open, validate};
use std::{fs, io, path::PathBuf};
use tempfile::TempDir;
use tokio_rusqlite::rusqlite::Connection;

fn initialize_test_db() -> io::Result<(PathBuf, TempDir)> {
    let temp = tempfile::tempdir()?;
    let database_path = temp.path().join("app.sqlite");

    initialize(&database_path).map_err(io::Error::other)?;

    Ok((database_path, temp))
}

#[test]
fn initialize_creates_database_and_schema_and_can_be_repeated() -> io::Result<()> {
    let (database_path, _temp) = initialize_test_db()?;
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
    let (database_path, _temp) = initialize_test_db()?;

    fs::remove_file(&database_path)?;

    let error =
        validate(&database_path).expect_err("validation should reject a missing SQLite file");

    assert_eq!(error.kind(), io::ErrorKind::NotFound);

    Ok(())
}

#[tokio::test]
async fn create_topic_inserts_topic() -> Result<(), Box<dyn std::error::Error>> {
    let (database_path, _temp) = initialize_test_db()?;
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

#[tokio::test]
async fn list_topics_returns_topics_in_creation_order() -> Result<(), Box<dyn std::error::Error>> {
    let (database_path, _temp) = initialize_test_db()?;
    let database = open(&database_path).await?;

    let first_topic = create_topic(
        &database,
        String::from("Rust Weekly"),
        String::from("Weekly Rust updates"),
    )
    .await?;

    let second_topic = create_topic(
        &database,
        String::from("AI News"),
        String::from("Daily AI updates"),
    )
    .await?;

    let topics = list_topics(&database).await?;
    assert_eq!(topics, vec![first_topic, second_topic]);

    Ok(())
}
