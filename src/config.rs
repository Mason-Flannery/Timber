use std::{
    fmt,
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

    pub fn save(&self) {
        let file = std::fs::File::create(Config::config_path()).expect("Failed to save config");
        let mut writer = BufWriter::new(file);
        let _ = writer.write_all(toml::to_string(&self).expect("").as_bytes());
    }

    pub fn config_path() -> PathBuf {
        let app_dirs = platform_dirs::AppDirs::new(Some("Timber"), true)
            .expect("Error: Could not find data directory");
        app_dirs.data_dir.join("config.toml")
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

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", toml::to_string_pretty(&self).unwrap())
    }
}

pub fn reset_config() -> Config {
    // Reset the config on disk and return a new config
    let config = Config::default();
    config.save();
    config
}
