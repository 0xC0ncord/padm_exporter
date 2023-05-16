use log::error;
use serde::Deserialize;
use std::cell::RefCell;

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
    auth_data: RefCell<AuthData>,
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
            host: host.to_string(),
            scheme: scheme.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            interval,
            auth_data: RefCell::new(AuthData::new()),
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    /// Log into the device and retrieve authentication data
    async fn authenticate(&self) -> Result<(), reqwest::Error> {
        let request_url = format!("https://{}/api/oauth/token?grant_type=password", self.host);
        let params = [("username", &self.username), ("password", &self.password)];

        let response = self.client.post(&request_url).form(&params).send().await;

        match response {
            Err(e) => {
                error!("Authentication failed on endpoint {}: {}", self.host(), e);
                Err(e)
            }
            Ok(r) => match r.json().await {
                Err(e) => {
                    error!(
                        "Malformed auth response from endpoint {}: {}",
                        self.host(),
                        e
                    );
                    Err(e)
                }
                Ok(j) => {
                    self.auth_data.replace(j);
                    Ok(())
                }
            },
        }
    }
    async fn raw_get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", &self.auth_data.borrow().access_token),
            )
            .send()
            .await
    }
    /// Do an authenticated GET request
    pub async fn do_get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}://{}{}", self.scheme, self.host, path);

        // Authenticate if never authenticated before
        if self.auth_data.borrow().is_empty() {
            self.authenticate().await?;
        }

        let response = self.raw_get(&url).await;
        match response {
            Ok(r) => match r.error_for_status() {
                Ok(r) => Ok(r),
                Err(err) => match err.status() {
                    Some(reqwest::StatusCode::UNAUTHORIZED) => {
                        // Authenticate again if needed
                        self.authenticate().await?;
                        Ok(self.raw_get(&url).await?)
                    }
                    // Otherwise just return the error
                    _ => Err(err),
                },
            },
            Err(err) => Err(err),
        }
    }
}
