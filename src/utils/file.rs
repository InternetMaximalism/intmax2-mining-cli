use std::{
    env, fs,
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::bail;

use super::errors::CLIError;

const PROJECT_ROOT_FILE: &str = ".mining-cli-root";

pub fn create_file_with_content(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CLIError::IoError(e))?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content)?;
    Ok(())
}

pub fn get_project_root() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    if current_dir.join(PROJECT_ROOT_FILE).exists() {
        return Ok(current_dir);
    }
    let mut path = env::current_exe()?;
    while !path.join(PROJECT_ROOT_FILE).exists() {
        if !path.pop() {
            bail!("Could not find project root");
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_file_with_content() {
        let path = Path::new("test.txt");
        let content = b"Hello, World!";
        create_file_with_content(path, content).unwrap();
        let read_content = fs::read(path).unwrap();
        assert_eq!(content, read_content.as_slice());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_get_project_root() {
        let path = get_project_root().unwrap();
        assert!(path.join(PROJECT_ROOT_FILE).exists());
    }
}
