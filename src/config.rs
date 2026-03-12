use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_min_duration_ms")]
    pub min_duration_ms: u64,
    #[serde(default = "default_deploy_command_prefixes")]
    pub deploy_command_prefixes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_duration_ms: default_min_duration_ms(),
            deploy_command_prefixes: default_deploy_command_prefixes(),
        }
    }
}

pub fn config_path(sounds_dir: &Path) -> PathBuf {
    sounds_dir.join("config.toml")
}

pub fn load_config(sounds_dir: &Path) -> Result<Config, Box<dyn Error>> {
    let path = config_path(sounds_dir);
    if !path.exists() {
        return Ok(Config::default());
    }

    let raw = fs::read_to_string(&path)?;
    Ok(toml::from_str(&raw)?)
}

pub fn serialize_default_config() -> Result<String, Box<dyn Error>> {
    Ok(toml::to_string_pretty(&Config::default())?)
}

fn default_min_duration_ms() -> u64 {
    1500
}

fn default_deploy_command_prefixes() -> Vec<String> {
    vec![
        "git push".to_string(),
        "pnpm deploy".to_string(),
        "npm publish".to_string(),
        "vercel --prod".to_string(),
    ]
}
