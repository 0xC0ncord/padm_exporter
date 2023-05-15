use serde_json;

use crate::padm_client::variables::{Variable, is_metric, unpack_variable};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub device_type: String,
    pub variables: Vec<Variable>,
}

pub fn load_all_from(json: &serde_json::Value) -> Result<Vec<Device>, std::io::Error> {
    let mut devices: Vec<Device> = Vec::new();

    for item in json["data"].as_array().unwrap() {
        'inner: for item in &item["attributes"].as_object() {
            if !is_metric(item) {
                continue;
            }
            for device in &mut devices {
                if device.id == item["device_id"].as_i64().unwrap() {
                    let variable = unpack_variable(item);
                    device.variables.push(variable);
                    break 'inner;
                }
            }

            let mut variables = Vec::new();
            let variable = unpack_variable(item);
            variables.push(variable);

            let id = item["device_id"].as_i64().unwrap();
            let name = item["device_name"].as_str().unwrap().to_string();
            let device_type = item["device_type"].as_str().unwrap().to_string();

            devices.push(Device {
                id,
                name,
                device_type,
                variables,
            });
        }
    }

    Ok(devices)
}
