use std::{fs, io, path::Path};

pub fn initialize(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("data"))?;
    fs::create_dir_all(path.join("episodes"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn initialize_creates_directories_and_can_be_repeated() -> std::io::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("workspace");
        super::initialize(&workspace)?;
        super::initialize(&workspace)?;
        assert!(workspace.is_dir());
        assert!(workspace.join("data").is_dir());
        assert!(workspace.join("episodes").is_dir());
        Ok(())
    }
}
