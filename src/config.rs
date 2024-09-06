use serde::Deserialize;
use std::fs;
use crate::email::EmailSettings;

#[derive(Deserialize)]
pub struct Config {
    pub folder_path: String,
    pub email_settings: EmailSettings,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}