use anyhow;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::config;
use crate::padm_client;

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    var_type: String,
    help: String,
    device_values: HashMap<String, String>,
}
impl Variable {
    pub fn new(name: String, var_type: String, help: String) -> Variable {
        Variable {
            name,
            var_type,
            help,
            device_values: HashMap::new(),
        }
    }
}

async fn get_devices_from(
    client: &padm_client::client::PADMClient,
) -> anyhow::Result<Vec<padm_client::device::Device>, anyhow::Error> {
    let json: Value = from_str(
        client
            .do_get("/api/variables")
            .await?
            .text()
            .await?
            .as_str(),
    )?;
    Ok(padm_client::device::load_all_from(&json).unwrap())
}

fn format_output_from_devices(devices: &Vec<padm_client::device::Device>) -> String {
    let mut body: String = String::new();
    let mut variables: Vec<Variable> = Vec::new();

    for device in devices {
        for var in &device.variables {
            let label = var.get("name").unwrap().to_string();
            let value = var.get("value").unwrap().to_string();

            if let Some(var) = variables.iter_mut().find(|x| x.name == label) {
                var.device_values.insert(device.name.to_owned(), value);
            } else {
                let var_type = var.get("type").unwrap().to_string();
                let help = var.get("help").unwrap().to_string();

                let mut var = Variable::new(label, var_type, help);
                var.device_values.insert(device.name.to_owned(), value);

                variables.push(var);
            }
        }
    }

    for var in variables {
        body.push_str(format!("# HELP {} {}\n", var.name, var.help).as_str());
        body.push_str(format!("# TYPE {} {}\n", var.name, var.var_type).as_str());

        for device in var.device_values {
            let (device, value) = device;
            body.push_str(format!("{}{{device=\"{}\"}} {}\n", var.name, device, value).as_str());
        }
    }
    body
}

pub async fn run(config: config::Config, body: Arc<Mutex<String>>) {
    let mut device_arcs = Vec::new();

    // Spawn client threads
    for endpoint in config.endpoints() {
        let client = padm_client::client::PADMClient::new(
            endpoint.host().as_str(),
            endpoint.scheme(),
            endpoint.tls_insecure(),
            endpoint.interval(),
            endpoint.username(),
            endpoint.password(),
        );

        let arc = Arc::new(Mutex::new(Vec::new()));
        let arc_clone = arc.clone();
        let current = thread::current();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move { client_run(client, arc_clone, current).await });
            loop {
                thread::park();
            }
        });

        device_arcs.push(arc);
    }

    loop {
        thread::park();

        let mut all_devices = Vec::new();
        for arc in &device_arcs {
            all_devices.append(&mut arc.lock().unwrap().to_owned());
        }
        let output = format_output_from_devices(&all_devices);
        *body.lock().unwrap() = output;
    }
}

async fn client_run(
    client: padm_client::client::PADMClient,
    devices_arc: Arc<Mutex<Vec<padm_client::device::Device>>>,
    main_thread: std::thread::Thread,
) {
    loop {
        let devices = get_devices_from(&client)
            .await
            .expect("Failed getting devices from client!");
        *devices_arc.lock().unwrap() = devices;
        main_thread.unpark();
        async_std::task::sleep(Duration::from_millis(client.interval() * 1000)).await;
    }
}
