use serde_json;

use crate::padm_client::variables::{is_metric, unpack_variable, Variable};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub device_type: String,
    pub variables: Vec<Variable>,
}

pub fn load_all_from(json: &serde_json::Value) -> Result<Vec<Device>, std::io::Error> {
    let mut devices: Vec<Device> = Vec::new();

    let items: Vec<&serde_json::Map<String, serde_json::Value>> = json["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["attributes"].as_object())
        .filter_map(|item| is_metric(&item).then_some(item))
        .collect();

    for item in items {
        match devices
            .iter_mut()
            .find(|device| device.id == item["device_id"].as_i64().unwrap())
        {
            Some(device) => {
                let variable = unpack_variable(item);
                device.variables.push(variable);
                continue;
            }
            None => (),
        }

        let variables = vec![unpack_variable(item)];
        let id = item["device_id"].as_i64().unwrap();
        let name = item["device_name"].as_str().unwrap().to_string();
        let device_type = item["device_type"].as_str().unwrap().to_string();

        devices.push(Device {
            id,
            name,
            device_type,
            variables,
        })
    }
    Ok(devices)
}
