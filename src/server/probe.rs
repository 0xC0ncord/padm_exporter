use actix_web::web::Data;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::config;
use crate::padm_client;

struct Variable {
    name: String,
    var_type: String,
    help: String,
    device_values: HashMap<String, String>,
}
impl Variable {
    pub fn new(name: String, var_type: String, help: String) -> Variable {
        return Variable {
            name,
            var_type,
            help,
            device_values: HashMap::new(),
        };
    }
}

async fn get_devices_from(clients: &mut Vec<padm_client::client::PADMClient>) -> Vec<padm_client::device::Device> {
    let mut devices: Vec<padm_client::device::Device> = Vec::new();
    for client in clients {
        let json: Value = from_str(client.do_get("/api/variables")
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .as_str()
        ).unwrap();
        devices.append(&mut padm_client::device::load_all_from(&json).unwrap());
    }
    return devices;
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
    return body;
}

pub fn run(config: &config::config::Config, body_mutex: Data<Mutex<String>>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        run_async(&config, body_mutex).await
    });
    loop {}
}

pub async fn run_async(config: &config::config::Config, body_mutex: Data<Mutex<String>>) {
    let mut clients: Vec<padm_client::client::PADMClient> = Vec::new();
    for endpoint in config.endpoints() {
        clients.push(padm_client::client::PADMClient::new(
            endpoint.host().as_str(),
            endpoint.scheme(),
            endpoint.tls_insecure(),
            endpoint.username(),
            endpoint.password()
        ).await);
    }

    loop {
        let devices = get_devices_from(&mut clients).await;
        let output = format_output_from_devices(&devices);
        *body_mutex.lock().unwrap() = output;
        async_std::task::sleep(Duration::from_millis(100 * 30)).await;
    }
}
