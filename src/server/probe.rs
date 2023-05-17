use log::error;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use indexmap::IndexMap;

use tokio::sync::mpsc;

use crate::config;
use crate::padm_client::{
    client::PADMClient,
    device::{load_all_from, Device},
};

#[derive(Debug, Clone)]
struct Metric {
    mtype: String,
    help: String,
    metrics: Vec<DeviceMetric>,
}

#[derive(Debug, Clone)]
struct DeviceMetric {
    device: String,
    value: String,
    labels: HashMap<String, String>,
}

async fn get_devices_from(client: &PADMClient) -> Result<Vec<Device>, anyhow::Error> {
    let response = client.do_get("/api/variables").await;
    match response {
        Err(e) => Err(e.into()),
        Ok(r) => match r.error_for_status() {
            Err(e) => Err(e.into()),
            Ok(r) => match r.text().await {
                Err(e) => Err(e.into()),
                Ok(s) => {
                    let json = serde_json::from_str(&s);
                    let devices = load_all_from(&json?);
                    match devices {
                        Ok(v) => Ok(v),
                        Err(e) => Err(e.into()),
                    }
                }
            },
        },
    }
}

fn format_output_from_devices(devices: &Vec<Device>) -> Result<String, std::io::Error> {
    let mut metrics: IndexMap<&str, Metric> = IndexMap::new();

    devices
        .iter()
        .map(|device| (&device.name, &device.variables))
        .for_each(|(name, variables)| {
            variables.iter().for_each(|variable| {
                let device_metric = DeviceMetric {
                    device: name.to_string(),
                    value: variable.get("value").to_string(),
                    labels: variable.labels().clone().unwrap_or(HashMap::new()),
                };

                let var_name = variable.get("name");
                metrics
                    .entry(var_name)
                    .and_modify(|m| m.metrics.push(device_metric.clone()))
                    .or_insert(Metric {
                        mtype: variable.get("type").to_string(),
                        help: variable.get("help").to_string(),
                        metrics: vec![device_metric],
                    });
            })
        });

    let body = metrics.iter().map(|(name, metric)| {
        let mut body = format!(
            "# HELP {} {}\n\
            #TYPE {} {}\n",
            name, metric.help, name, metric.mtype
        )
        .to_string();
        let m_str = metric.metrics.iter().map(|device_metric| {
            let labels = device_metric
                .labels
                .iter()
                .map(|(k, v)| format!(",{}=\"{}\"", k, v));
            let mut inner = format!("device=\"{}\"", device_metric.device).to_string();
            inner.push_str(labels.collect::<String>().as_str());
            format!("padm_{}{{{}}} {}\n", name, inner, device_metric.value)
        });
        body.push_str(m_str.collect::<String>().as_str());
        body
    });

    Ok(body.collect())
}

pub async fn run(config: config::Config, body: Arc<Mutex<String>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<(Vec<Device>, usize)>();

    config
        .endpoints()
        .iter()
        .map(|e| {
            PADMClient::new(
                e.host().as_str(),
                e.scheme(),
                e.tls_insecure(),
                e.interval(),
                e.username(),
                e.password(),
            )
        })
        .enumerate()
        // Spawn client threads
        .for_each(|(idx, client)| {
            let thread_tx = tx.clone();
            tokio::task::spawn(client_run(client, idx, thread_tx));
        });

    let mut devices_vec: Vec<Vec<Device>> = Vec::with_capacity(config.endpoints().len());

    loop {
        // Blocks until data
        let (d, i) = rx.recv().await.unwrap();
        devices_vec[i] = d;

        let devices = devices_vec.concat();
        match format_output_from_devices(&devices) {
            Ok(output) => *body.lock().unwrap() = output,
            Err(e) => error!("Failed formatting metrics output: {}", e),
        }
    }
}

async fn client_run(
    client: PADMClient,
    index: usize,
    thread_tx: mpsc::UnboundedSender<(Vec<Device>, usize)>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(client.interval()));
    loop {
        interval.tick().await;
        match get_devices_from(&client).await {
            Ok(d) => thread_tx.send((d, index)).unwrap(),
            Err(e) => error!(
                "Failed getting devices from client {}: {}",
                &client.host(),
                e
            ),
        }
    }
}
