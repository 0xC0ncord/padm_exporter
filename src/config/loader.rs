use anyhow::{Context, Result};

use crate::config::Config;

pub fn load_from_file(file_path: &str) -> Result<Config> {
    let contents = std::fs::read_to_string(file_path).context("error reading config file")?;
    let config: Config =
        serde_yml::from_str(&contents).context("yaml parsing failed")?;
    Ok(config)
}
