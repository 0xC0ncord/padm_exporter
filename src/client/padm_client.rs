use anyhow::{Context, Result, anyhow};
use log::error;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread::Thread;
use std::time::Duration;

use crate::client::auth::AuthData;
use crate::metrics::{MetricsRegistry, PADMMetric};
use crate::target::ApiResponse;

/*
* Client for interacting with PADM targets
*/
pub struct PADMClient {
    client: reqwest::Client,
    addr: String,
    url: String,
    interval: u64,
    username: String,
    password: String,
    auth_data: RefCell<AuthData>,
    api_response: ApiResponse,
    registry: Arc<MetricsRegistry>,
}
impl PADMClient {
    pub fn new(
        addr: String,
        url: String,
        tls_insecure: bool,
        interval: u64,
        username: &str,
        password: &str,
        registry: Arc<MetricsRegistry>,
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
            addr,
            url: url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            interval,
            auth_data: RefCell::new(AuthData::new()),
            api_response: ApiResponse::new(),
            registry,
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    /// Log into the target and retrieve authentication data
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
                    error!("Malformed auth response from endpoint {}: {}", self.url, e);
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
    async fn do_get(&self, path: &str) -> Result<reqwest::Response> {
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
    /// Probe the device
    async fn probe(&mut self) -> Result<()> {
        self.api_response = self
            .do_get("/api/variables")
            .await
            .context("Network error")?
            .json()
            .await
            .context("Failed to deserialize JSON")?;
        Ok(())
    }
    /// Update metrics from the ApiResponse
    async fn update_metrics(&mut self) {
        for var in self.api_response.data.iter() {
            let attr = var.attributes.clone();
            if let Some(metric) = PADMMetric::from_label(&attr.label) {
                let key = metric.to_metric_key();
                let value: f64 = match attr.raw_value.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        log::warn!(
                            "Could not parse value '{}' for label '{}'",
                            attr.raw_value,
                            attr.label
                        );
                        continue;
                    }
                };
                let mut label_values = vec![attr.device_name];
                if key.labels.len() > 1 {
                    label_values.push(attr.value);
                }
                let label_refs: Vec<&str> = label_values.iter().map(String::as_str).collect();
                if let Err(e) = self.registry.update_metric(&key, &label_refs, value) {
                    log::error!("Failed to update or register metric {}: {}", key.name, e);
                    continue;
                }
            }
        }
    }
    /// Run this client
    pub async fn run(&mut self, main_thread: Thread) {
        loop {
            if let Err(e) = self.probe().await {
                log::error!("Error from client {}: {e}", self.addr);
            } else {
                self.update_metrics().await;
            }
            main_thread.unpark();
            async_std::task::sleep(Duration::from_secs(self.interval())).await;
        }
    }
}
