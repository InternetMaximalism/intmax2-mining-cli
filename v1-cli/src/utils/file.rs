use std::{fs, io::Write as _, path::Path};

pub fn create_file_with_content(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content)?;
    Ok(())
}
