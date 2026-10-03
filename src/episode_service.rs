use crate::{episode::Episode, storage, workspace::WorkspacePaths};

pub async fn create_episode(
    database: &tokio_rusqlite::Connection,
    workspace_paths: &WorkspacePaths,
    topic_id: i64,
) -> Result<Episode, Box<dyn std::error::Error + Send + Sync>> {
    let episode = storage::create_episode(database, topic_id).await?;
    workspace_paths.create_episode_dir(episode.topic_id, episode.id)?;
    Ok(episode)
}

#[cfg(test)]
mod tests;
