use serde_json;

use indexmap::IndexMap;

use crate::padm_client::variables::{is_metric, unpack_variable, Variable};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub device_type: String,
    pub variables: Vec<Variable>,
}

pub fn load_all_from(json: &serde_json::Value) -> Result<Vec<Device>, std::io::Error> {
    let data = json["data"].as_array().unwrap();
    // micro-optimization
    let devices: IndexMap<i64, Device> = IndexMap::with_capacity(data.len());
    let items = data
        .iter()
        .filter_map(|i| i["attribute"].as_object())
        .filter(|i| is_metric(&i));

    for item in items {
        let id = item["device_id"].as_i64().unwrap();
        let variable = unpack_variable(item);
        devices
            .entry(id)
            .and_modify(|device| device.variables.push(variable))
            .or_insert_with(|| {
                let name = item["device_name"].as_str().unwrap().to_string();
                let device_type = item["device_type"].as_str().unwrap().to_string();
                Device {
                    id,
                    name,
                    device_type,
                    variables: vec![variable],
                }
            });
    }
    Ok(devices.into_values().collect())
}
