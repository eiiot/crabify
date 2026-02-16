use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub client_id: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // Load .env file if present (won't override existing env vars)
        let _ = dotenvy::dotenv();

        // First check environment variable
        if let Ok(client_id) = std::env::var("SPOTIFY_CLIENT_ID") {
            let client_id = client_id.trim().to_string();
            if !client_id.is_empty() {
                return Ok(Self { client_id });
            }
        }

        // Then check config file
        let config_path = Self::config_file_path()?;
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            let config: AppConfig = serde_json::from_str(&contents)
                .with_context(|| "Failed to parse config file")?;
            return Ok(config);
        }

        anyhow::bail!(
            "Spotify Client ID not found.\n\
             Set SPOTIFY_CLIENT_ID environment variable or create config at {}",
            config_path.display()
        )
    }

    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("crabify");
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }
}
