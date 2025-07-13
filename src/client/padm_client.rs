use anyhow::{anyhow, Result};
use log::error;
use std::cell::RefCell;

use crate::client::auth::AuthData;

/*
* Client for interacting with PADM devices
*/
pub struct PADMClient {
    client: reqwest::Client,
    url: String,
    interval: u64,
    username: String,
    password: String,
    auth_data: RefCell<AuthData>,
}
impl PADMClient {
    pub fn new(
        url: &str,
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
            url: url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            interval,
            auth_data: RefCell::new(AuthData::new()),
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Log into the device and retrieve authentication data
    async fn authenticate(&self) -> Result<()> {
        let request_url = format!("https://{}/api/oauth/token?grant_type=password", self.url);
        let params = [("username", &self.username), ("password", &self.password)];

        let response = self.client.post(&request_url).form(&params).send().await;

        match response {
            Err(e) => {
                error!("Authentication failed on endpoint {}: {}", self.url, e);
                Err(anyhow!(e))
            }
            Ok(r) => match r.json().await {
                Err(e) => {
                    error!(
                        "Malformed auth response from endpoint {}: {}",
                        self.url,
                        e
                    );
                    Err(anyhow!(e))
                }
                Ok(j) => {
                    self.auth_data.replace(j);
                    Ok(())
                }
            },
        }
    }
    async fn raw_get(&self, url: &str) -> Result<reqwest::Response> {
        self.client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", &self.auth_data.borrow().access_token),
            )
            .send()
            .await
            .map_err(|e| anyhow!(e))
    }
    /// Do an authenticated GET request
    pub async fn do_get(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}/{}", self.url, path);

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
                    _ => Err(anyhow!(err)),
                },
            },
            Err(err) => Err(err),
        }
    }
}
