use anyhow::{Context, Result, anyhow};
use log::error;
use std::thread::Thread;
use std::time::Duration;
use tokio::sync::RwLock;

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
    auth_data: RwLock<AuthData>,
    api_response: RwLock<ApiResponse>,
    registry: RwLock<MetricsRegistry>,
}
impl PADMClient {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        addr: String,
        url: String,
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
            addr,
            url: url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            interval,
            auth_data: RwLock::new(AuthData::new()),
            api_response: RwLock::new(ApiResponse::new()),
            registry: RwLock::new(MetricsRegistry::new()),
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    pub fn registry(&self) -> &RwLock<MetricsRegistry> {
        &self.registry
    }
    /// Log into the target and retrieve authentication data
    async fn authenticate(&self) -> Result<()> {
        let request_url = self.url.to_string() + "/api/oauth/token?grant_type=password";
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
                    let mut auth_data = self.auth_data.write().await;
                    *auth_data = j;
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
                format!("Bearer {}", self.auth_data.read().await.access_token),
            )
            .send()
            .await
            .map_err(|e| anyhow!(e))
    }
    /// Do an authenticated GET request
    async fn do_get(&self, path: &str) -> Result<reqwest::Response> {
        let url = self.url.to_string() + path;

        // Authenticate if never authenticated before
        if self.auth_data.read().await.is_empty() {
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
    async fn probe(&self) -> Result<()> {
        let mut api_data = self.api_response.write().await;
        let response_data: ApiResponse = self
            .do_get("/api/variables")
            .await
            .context("Network error")?
            .json()
            .await
            .context("Failed to deserialize JSON")?;
        *api_data = response_data;
        Ok(())
    }
    /// Update metrics from the ApiResponse
    async fn update_metrics(&self) {
        for var in self.api_response.read().await.data.iter() {
            let attr = var.attributes.clone();
            if let Some(metric) = PADMMetric::from_label(&attr.label) {
                let key = metric.to_metric_key();

                if key.is_enum && !attr.enum_values.is_empty() {
                    let current_value = attr.value.as_str();
                    for variant in &attr.enum_values {
                        let metric_value = if variant.name == current_value {
                            1.0
                        } else {
                            0.0
                        };
                        let label_refs = [&*attr.device_name, &*variant.name];
                        if let Err(e) = self.registry.write().await.update_metric(&key, &label_refs, metric_value)
                        {
                            log::error!("Failed to update or register metric {}: {}", key.name, e);
                        }
                    }
                } else {
                    let raw_value: f64 = match attr.get_raw() {
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

                    let label_refs: Vec<&str> = if key.labels.len() > 1 {
                        vec![&*attr.device_name, &*attr.value]
                    } else {
                        vec![&*attr.device_name]
                    };
                    if let Err(e) = self.registry.write().await.update_metric(&key, &label_refs, raw_value) {
                        log::error!("Failed to update or register metric {}: {}", key.name, e);
                    }
                }
            }
        }
    }
    /// Run this client
    pub async fn run(&self, main_thread: Thread) {
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
