use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    ip: String,
    port: Option<u16>,
    endpoints: Vec<Endpoint>,
}
impl Config {
    pub fn ip(&self) -> &str {
        self.ip.as_str()
    }
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(8000)
    }
    pub fn endpoints(&self) -> &Vec<Endpoint> {
        &self.endpoints
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.ip(), self.port())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    ip: String,
    port: Option<u16>,
    scheme: Option<String>,
    tls_insecure: Option<bool>,
    interval: Option<u64>,
    username: String,
    password: String,
}
impl Endpoint {
    pub fn ip(&self) -> &str {
        self.ip.as_str()
    }
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(443)
    }
    pub fn scheme(&self) -> &str {
        match &self.scheme {
            Some(s) => s,
            None => "https"
        }
    }
    pub fn tls_insecure(&self) -> bool {
        self.tls_insecure.unwrap_or(false)
    }
    pub fn interval(&self) -> u64 {
        self.interval.unwrap_or(30)
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn host(&self) -> String {
        format!("{}:{}", self.ip(), self.port())
    }
}

pub fn load_config_from_file(file_path: &str) -> Result<Config, std::io::Error> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)
        .expect("Failed parsing toml config");
    Ok(config)
}
