use serde::Deserialize;
use std::fs;
use toml;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    ip: String,
    port: Option<u16>,
    endpoints: Vec<Endpoint>,
}
impl Config {
    pub fn ip(&self) -> &str {
        return self.ip.as_str();
    }
    pub fn port(&self) -> u16 {
        return match self.port {
            Some(p) => p,
            None => 8000,
        };
    }
    pub fn endpoints(&self) -> &Vec<Endpoint> {
        return &self.endpoints;
    }

    pub fn bind_address(&self) -> String {
        return format!("{}:{}", self.ip(), self.port());
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    ip: String,
    port: Option<u16>,
    scheme: Option<String>,
    tls_insecure: Option<bool>,
    interval: Option<u32>,
    username: String,
    password: String,
}
impl Endpoint {
    pub fn ip(&self) -> &str {
        return &self.ip.as_str();
    }
    pub fn port(&self) -> u16 {
        return match self.port {
            Some(p) => p,
            None => 443,
        };
    }
    pub fn scheme(&self) -> &str {
        return match &self.scheme {
            Some(s) => &s,
            None => &"https",
        };
    }
    pub fn tls_insecure(&self) -> bool {
        return match self.tls_insecure {
            Some(b) => b,
            None => false,
        };
    }
    pub fn interval(&self) -> u32 {
        return match self.interval {
            Some(i) => i,
            None => 30,
        };
    }
    pub fn username(&self) -> &str {
        return &self.username;
    }
    pub fn password(&self) -> &str {
        return &self.password;
    }

    pub fn host(&self) -> String {
        return format!("{}:{}", self.ip(), self.port());
    }
}

pub fn load_config_from_file(file_path: &str) -> Result<Config, std::io::Error> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)
        .expect("Failed parsing toml config");
    Ok(config)
}
