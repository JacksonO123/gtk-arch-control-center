use std::{
    collections,
    fs::{self, File},
    io::Write,
};

use crate::utils;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    ErrorGettingConfigDir,
    ConfigDirNotFound,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ErrorGettingConfigDir => write!(f, "Error getting config dir"),
            ConfigError::ConfigDirNotFound => write!(f, "Config dir not found"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub struct SavedState {
    pub wifi: bool,
    pub bluetooth: bool,
    pub sunset: bool,
}

impl SavedState {
    pub fn get_string(&self) -> String {
        format!(
            "wifi={}\nbluetooth={}\nsunset={}\n",
            self.wifi, self.bluetooth, self.sunset
        )
    }
}

fn parse_config(config_data: String) -> collections::HashMap<String, String> {
    config_data
        .split('\n')
        .filter_map(|line| {
            let parts: Vec<_> = line.splitn(2, '=').collect();

            if parts.len() < 2 {
                return None;
            }

            Some((parts[0].to_string(), parts[1].to_string()))
        })
        .collect::<collections::HashMap<String, String>>()
}

pub fn read_config() -> Result<SavedState, Box<dyn std::error::Error>> {
    let config_path =
        utils::get_config_dir().ok_or(Box::new(ConfigError::ErrorGettingConfigDir))?;

    if !fs::exists(&config_path)? {
        return Err(Box::new(ConfigError::ConfigDirNotFound));
    }

    let file = fs::read_to_string(config_path)?;
    let parsed = parse_config(file);

    let config_includes_as = move |key, value| match parsed.get(key) {
        Some(val) => val == value,
        None => false,
    };

    Ok(SavedState {
        wifi: config_includes_as("wifi", "true"),
        bluetooth: config_includes_as("bluetooth", "true"),
        sunset: config_includes_as("sunset", "true"),
    })
}

pub fn write_config_file(config: &SavedState) -> Result<(), Box<dyn std::error::Error>> {
    let base_config_path =
        utils::get_config_base_dir().ok_or(Box::new(ConfigError::ErrorGettingConfigDir))?;

    if !fs::exists(&base_config_path)? {
        fs::create_dir(&base_config_path)?;
    }

    let config_path =
        utils::get_config_dir().ok_or(Box::new(ConfigError::ErrorGettingConfigDir))?;

    let config_str = config.get_string();
    let mut file = File::create(config_path)?;
    file.write_all(config_str.as_bytes())?;

    Ok(())
}
