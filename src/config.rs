use std::path::PathBuf;

use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
}

impl Config {
    pub fn load() -> Option<Self> {
        let app_dirs = platform_dirs::AppDirs::new(Some("Timber"), true)?;
        let config_path = app_dirs.data_dir.join("config.toml");
        let content = std::fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    }
}

impl Default for Config {
    fn default() -> Self {
        // Get appdata dir
        let app_dirs =
            platform_dirs::AppDirs::new(Some("Timber"), true).expect("Failed to get directories");
        let db_path = app_dirs.data_dir.join("timber.db");

        Config {
            database_path: db_path,
        }
    }
}
