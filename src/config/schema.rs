use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    ip: String,
    port: Option<u16>,
    log_level: Option<String>,
    targets: Vec<Target>,
}
impl Config {
    pub fn ip(&self) -> &str {
        self.ip.as_str()
    }
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(8000)
    }
    pub fn log_level(&self) -> &str {
        match &self.log_level {
            Some(s) => s,
            None => "info"
        }
    }
    pub fn targets(&self) -> &Vec<Target> {
        &self.targets
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.ip(), self.port())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    host: String,
    port: Option<String>,
    scheme: Option<String>,
    tls_insecure: Option<bool>,
    interval: Option<u64>,
    username: String,
    password: String,
}
impl Target {
    pub fn addr(&self) -> String {
        let mut addr = self.host.to_string();
        if let Some(port) = &self.port {
            addr.push_str(&format!(":{port}"));
        }
        addr
    }
    pub fn url(&self) -> String {
        let scheme = self.scheme.as_deref().unwrap_or("https");
        let mut url = format!("{}://{}", scheme, self.host);
        if let Some(port) = &self.port {
            url.push_str(&format!(":{port}"));
        }
        url
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
}
