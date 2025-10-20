use std::fs;

use anyhow::Context as _;

use super::file::get_data_path;

/// Removes the cached GitHub responses stored under `~/.mining-cli/github_cache`.
/// Returns `Ok(true)` when the cache directory existed and was removed.
pub fn clear_github_cache() -> anyhow::Result<bool> {
    let cache_dir = get_data_path()?.join("github_cache");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("failed to remove cache directory: {:?}", cache_dir))?;
        Ok(true)
    } else {
        Ok(false)
    }
}
