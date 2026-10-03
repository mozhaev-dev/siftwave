use super::{CURRENT_SCHEMA_VERSION, WorkspaceConfig, WorkspacePaths, initialize, validate};
use std::{fs, io};

#[test]
fn initialize_creates_directories_and_can_be_repeated() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let workspace_paths = WorkspacePaths::new(&temp.path().join("workspace"));

    initialize(&workspace_paths.root)?;
    initialize(&workspace_paths.root)?;

    assert!(workspace_paths.root.is_dir());
    assert!(workspace_paths.data.is_dir());
    assert!(workspace_paths.episodes.is_dir());
    assert!(workspace_paths.database.is_file());

    let config_content = fs::read_to_string(&workspace_paths.config)?;
    let config: WorkspaceConfig = toml::from_str(&config_content).map_err(io::Error::other)?;

    assert_eq!(
        config,
        WorkspaceConfig {
            schema_version: CURRENT_SCHEMA_VERSION
        }
    );

    validate(&workspace_paths.root)?;

    Ok(())
}

#[test]
fn validate_rejects_uninitialized_directory() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let error = validate(temp.path()).expect_err("validation should fail without a config");

    assert_eq!(error.kind(), io::ErrorKind::NotFound);

    Ok(())
}

#[test]
fn validate_rejects_unsupported_schema_version() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let workspace_paths = WorkspacePaths::new(&temp.path().join("workspace"));

    initialize(&workspace_paths.root)?;

    fs::write(&workspace_paths.config, "schema_version = 999\n")?;

    let error = validate(&workspace_paths.root)
        .expect_err("validation should reject an unsupported schema version");

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);

    Ok(())
}

#[test]
fn episode_dir_returns_path_under_topic_directory() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let workspace_paths = WorkspacePaths::new(&temp.path().join("workspace"));
    let topic_id = 123;
    let episode_id = 456;

    let path = workspace_paths.episode_dir(topic_id, episode_id);

    let expected = workspace_paths
        .episodes
        .join(topic_id.to_string())
        .join(episode_id.to_string());

    assert_eq!(path, expected);

    Ok(())
}

#[test]
fn create_episode_dir_creates_directory_and_can_be_repeated() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let workspace_paths = WorkspacePaths::new(&temp.path().join("workspace"));

    let topic_id = 123;
    let episode_id = 456;

    let path_1 = workspace_paths.create_episode_dir(topic_id, episode_id)?;
    let path_2 = workspace_paths.create_episode_dir(topic_id, episode_id)?;

    assert_eq!(&path_1, &path_2);
    assert!(path_1.is_dir());

    Ok(())
}
