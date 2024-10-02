use std::{
    fs::{self, File},
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::Context as _;

pub fn get_data_path() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    Ok(home_dir.join(".mining-cli"))
}

pub fn create_data_dir() -> anyhow::Result<()> {
    let data_path = get_data_path()?;
    fs::create_dir_all(&data_path)
        .with_context(|| format!("Failed to create directory: {:?}", data_path))?;
    Ok(())
}

pub fn create_file_with_content(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    let mut file =
        File::create(path).with_context(|| format!("Failed to create file: {:?}", path))?;

    file.write_all(content)
        .with_context(|| format!("Failed to write content to file: {:?}", path))?;
    Ok(())
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
}
