use anyhow;
use reqwest;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: String,
    pub msg: String,
}
impl AuthData {
    pub fn new() -> AuthData {
        AuthData {
            access_token: String::new(),
            refresh_token: String::new(),
            msg: String::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.access_token.is_empty() || self.refresh_token.is_empty() || self.msg.is_empty()
    }
}

/*
* Client for interacting with PADM devices
*/
pub struct PADMClient {
    client: reqwest::Client,
    host: String,
    scheme: String,
    interval: u64,
    username: String,
    password: String,
    auth_data: Arc<Mutex<AuthData>>,
}
impl PADMClient {
    pub fn new(
        host: &str,
        scheme: &str,
        tls_insecure: bool,
        interval: u64,
        username: &str,
        password: &str,
    ) -> PADMClient {
        let mut client_builder = reqwest::Client::builder();
        // Disable SSL verification if asked
        if tls_insecure {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        // Get a new reqwest client
        let client = client_builder.build().unwrap();

        PADMClient {
            client,
            host: String::from(host),
            scheme: String::from(scheme),
            username: username.to_string(),
            password: password.to_string(),
            interval,
            auth_data: Arc::new(Mutex::new(AuthData::new())),
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    /// Log into the device and retrieve authentication data
    async fn authenticate(&self) -> anyhow::Result<()> {
        let request_url = format!("https://{}/api/oauth/token?grant_type=password", self.host);
        let params = [("username", &self.username), ("password", &self.password)];

        let auth_data: AuthData = self
            .client
            .post(&request_url)
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        *self.auth_data.lock().unwrap() = auth_data;
        Ok(())
    }
    async fn raw_get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", &self.auth_data.lock().unwrap().access_token),
            )
            .send()
            .await
    }
    /// Do an authenticated GET request
    pub async fn do_get(&self, path: &str) -> anyhow::Result<reqwest::Response> {
        let url = format!("{}://{}{}", self.scheme, self.host, path);

        // Authenticate if never authenticated before
        if self.auth_data.lock().unwrap().is_empty() {
            self.authenticate().await?;
        }

        let response = self.raw_get(&url).await;
        match response {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(code) = e.status() {
                    // Authenticate again if needed
                    if code == reqwest::StatusCode::FORBIDDEN {
                        self.authenticate().await?;
                        return Ok(self.raw_get(&url).await?);
                    }
                }
                // Otherwise just return the error
                Err(anyhow::Error::new(e))
            }
        }
    }
}
