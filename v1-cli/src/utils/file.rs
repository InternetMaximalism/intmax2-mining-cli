use std::{fs, io::Write as _, path::Path};

use super::errors::CLIError;

pub fn create_file_with_content(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CLIError::IoError(e))?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content)?;
    Ok(())
}
