use anyhow::Result;
use std::fs;

use crate::config::Config;

pub fn load_from_file(file_path: &str) -> Result<Config> {
    let config: Config =
        toml::from_str(&fs::read_to_string(file_path)?).expect("Failed parsing toml config");
    Ok(config)
}
