use super::create_episode;
use crate::{
    storage,
    workspace::{self, WorkspacePaths},
};

#[tokio::test]
async fn create_episode_creates_episode_directory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temp = tempfile::tempdir()?;
    workspace::initialize(temp.path())?;

    let workspace_paths = WorkspacePaths::new(temp.path());
    let database = storage::open(&workspace_paths.database).await?;
    let topic = storage::create_topic(
        &database,
        String::from("Rust Weekly"),
        String::from("Weekly Rust updates"),
    )
    .await?;

    let episode = create_episode(&database, &workspace_paths, topic.id).await?;
    let episode_dir = workspace_paths.episode_dir(episode.topic_id, episode.id);

    assert!(episode_dir.is_dir());

    Ok(())
}
