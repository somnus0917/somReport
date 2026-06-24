use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "som-report";

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
}

pub fn app_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(APP_NAME)
}

pub fn db_path() -> PathBuf {
    app_data_dir().join("som-report.db")
}

pub fn temp_image_dir() -> PathBuf {
    app_cache_dir().join("temp_images")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    ensure_dirs_in(app_data_dir(), app_cache_dir())
}

fn ensure_dirs_in(data_dir: PathBuf, cache_dir: PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    fs::create_dir_all(&cache_dir)?;
    let temp = cache_dir.join("temp_images");
    fs::create_dir_all(&temp)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

pub fn cleanup_temp_files() -> std::io::Result<()> {
    let dir = temp_image_dir();
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

pub fn clear_cache() -> std::io::Result<()> {
    let dir = app_cache_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    ensure_dirs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_are_valid() {
        let data = app_data_dir();
        let cache = app_cache_dir();
        let db = db_path();
        let temp = temp_image_dir();

        assert!(data.to_str().unwrap().contains(APP_NAME));
        assert!(cache.to_str().unwrap().contains(APP_NAME));
        assert!(db.to_str().unwrap().ends_with("som-report.db"));
        assert!(temp.to_str().unwrap().contains("temp_images"));
    }

    #[test]
    fn test_ensure_dirs_creates_directories() {
        let root = std::env::temp_dir().join(format!("som-report-paths-{}", uuid::Uuid::new_v4()));
        let data = root.join("data");
        let cache = root.join("cache");
        let temp = cache.join("temp_images");

        ensure_dirs_in(data.clone(), cache.clone()).unwrap();

        assert!(temp.exists());
        assert!(data.exists());
        assert!(cache.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&temp).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }

        fs::remove_dir_all(root).unwrap();
    }
}
