use anyhow::{Context, Result, anyhow};
use std::thread::Thread;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};

use crate::client::auth::AuthData;
use crate::metrics::{MetricKey, MetricsRegistry, PADMMetric};
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
    tracked_devices: Vec<String>,
    auth_data: RwLock<AuthData>,
    api_response: RwLock<Option<ApiResponse>>,
    registry: RwLock<MetricsRegistry>,
    ready: Notify,
    probe: Notify,
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
        tracked_devices: Vec<String>,
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
            tracked_devices,
            interval,
            auth_data: RwLock::new(AuthData::new()),
            api_response: RwLock::new(None),
            registry: RwLock::new(MetricsRegistry::new()),
            ready: Notify::new(),
            probe: Notify::new(),
        }
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    pub fn is_manual(&self) -> bool {
        self.interval == 0
    }
    pub fn registry(&self) -> &RwLock<MetricsRegistry> {
        &self.registry
    }
    pub fn ready(&self) -> &Notify {
        &self.ready
    }
    pub fn probe(&self) -> &Notify {
        &self.probe
    }
    pub async fn is_ready(&self) -> bool {
        !self.registry.read().await.registry.gather().is_empty() && !self.is_manual()
    }
    /// Log into the target and retrieve authentication data
    async fn authenticate(&self) -> Result<()> {
        log::debug!("{}: authenticating with target", self.addr);

        let request_url = self.url.to_string() + "/api/oauth/token?grant_type=password";
        let params = [("username", &self.username), ("password", &self.password)];

        let response_json = self
            .client
            .post(&request_url)
            .form(&params)
            .send()
            .await
            .context("authentication failed")?
            .json()
            .await
            .context("malformed auth response")?;

        log::debug!("{}: authentication successful", self.addr);
        let mut auth_data = self.auth_data.write().await;
        *auth_data = response_json;
        Ok(())
    }
    async fn raw_get(&self, url: &str) -> Result<reqwest::Response> {
        log::debug!("{}: raw get to url {}", self.addr, url);
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
            self.authenticate()
                .await
                .with_context(|| format!("{}: failed to send web request", self.addr))?;
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
    async fn do_probe(&self) -> Result<()> {
        let mut api_data = self.api_response.write().await;
        let response_data: ApiResponse = self
            .do_get("/api/variables")
            .await
            .context("Network error")?
            .json()
            .await
            .context("Failed to deserialize JSON")?;

        log::debug!("{}: got ApiResponse:\n{:?}", self.addr, response_data);
        *api_data = Some(response_data);

        Ok(())
    }
    async fn write_metric(&self, key: &MetricKey, label_refs: &Vec<&str>, raw_value: f64) {
        if let Err(e) = self
            .registry
            .write()
            .await
            .update_metric(key, label_refs, raw_value)
        {
            log::error!("Failed to update or register metric {}: {}", key.name, e);
        }
    }
    /// Update metrics from the ApiResponse
    async fn update_metrics(&self) {
        let do_notify = !self.is_ready().await;

        // Update metrics from target
        let guard = self.api_response.read().await;
        let response_data = guard.as_ref().unwrap();
        for var in response_data.data.iter() {
            let attr = var.attributes.clone();

            if PADMMetric::is_ignored(&attr.label) {
                log::debug!("{}: variable '{}' ignored", self.addr, attr.label);
                continue;
            }

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
                        let label_refs =
                            [&*attr.device_name, &sanitize_label(variant.name.to_owned())];
                        if let Err(e) = self.registry.write().await.update_metric(
                            &key,
                            &label_refs,
                            metric_value,
                        ) {
                            log::error!(
                                "Failed to update or register metric '{}': {}",
                                key.name,
                                e
                            );
                        }
                    }
                } else {
                    let raw_value: f64 = match attr.parse_raw() {
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

                    let value = &sanitize_label(attr.value);
                    let label_refs: Vec<&str> = if key.labels.len() > 1 {
                        vec![&*attr.device_name, value]
                    } else {
                        vec![&*attr.device_name]
                    };

                    log::debug!(
                        "{}: updating metric '{}' with value {}",
                        self.addr,
                        key.name,
                        raw_value
                    );
                    self.write_metric(&key, &label_refs, raw_value).await;
                }
            } else {
                log::debug!(
                    "{}: variable '{}' left unmapped",
                    self.addr,
                    var.attributes.label
                );
            }
        }

        // Update tracked device status metrics
        for device in self.tracked_devices.iter() {
            let found = f64::from(
                response_data
                    .data
                    .iter()
                    .any(|var| var.attributes.device_name == *device),
            );

            log::debug!(
                "{}: updating tracked device '{}' metric with value {}",
                self.addr,
                device,
                found
            );

            if let Some(metric) = PADMMetric::from_label("Device Up") {
                let key = metric.to_metric_key();
                self.write_metric(&key, &vec![device], found).await;
            } else {
                log::error!(
                    "{}: failed updating tracked device '{}' metric",
                    self.addr,
                    device
                );
            }
        }

        if do_notify {
            self.ready.notify_waiters();
        }
    }
    /// Run this client
    pub async fn run(&self, main_thread: Thread) {
        loop {
            if self.is_manual() {
                self.probe.notified().await
            }

            if let Err(e) = self.do_probe().await {
                log::error!("Error from client {}: {e}", self.addr);
            } else {
                self.update_metrics().await;
            }

            main_thread.unpark();

            if !self.is_manual() {
                async_std::task::sleep(Duration::from_secs(self.interval())).await;
            }
        }
    }
}

fn sanitize_label(label: String) -> String {
    label
        .to_lowercase()
        .replace([' ', '-', '(', ')'], "_")
        .replace("__", "_")
        .trim_end_matches("_")
        .to_string()
}
