use std::{sync::Mutex, collections::HashMap};
use once_cell::sync::Lazy;
use serde_json::Map;

static PADM_VARIABLE_MAP: Lazy<Mutex<HashMap<&str, HashMap<&str, &str>>>> = Lazy::new(|| {
    let map = HashMap::from([
        ("Return Air Temperature (C)", HashMap::from([
            ("name", "return_air_temperature"),
            ("type", "gauge"),
            ("help", "Temperature of the return air in celsius"),
        ])),
        ("Remote Set Point Temperature (C)", HashMap::from([
            ("name", "remote_set_point_temperature"),
            ("type", "gauge"),
            ("help", "Set point temperature of the remote sensor"),
        ])),
        ("Temperature (C)", HashMap::from([
            ("name", "temperature"),
            ("type", "gauge"),
            ("help", "Current detected temperature"),
        ])),
        ("Contact Input Count", HashMap::from([
            ("name", "contact_input_count"),
            ("type", "counter"),
            ("help", "Unknown"),
        ])),
        ("Contact Output Count", HashMap::from([
            ("name", "contact_output_count"),
            ("type", "counter"),
            ("help", "Unknown"),
        ])),
        ("Temperature (C) Low Critical Threshold", HashMap::from([
            ("name", "temp_low_crit_threshold"),
            ("type", "gauge"),
            ("help", "Critically low temperature"),
        ])),
        ("Temperature (C) High Critical Threshold", HashMap::from([
            ("name", "temp_high_crit_threshold"),
            ("type", "gauge"),
            ("help", "Critically high temperature"),
        ])),
    ]);
    Mutex::new(map)
});

pub fn unpack_variable(data: &Map<String, serde_json::Value>) -> HashMap<String, String> {
    let mut m: HashMap<String, String> = HashMap::new();
    let map = PADM_VARIABLE_MAP.lock().unwrap();

    let extract = |field: &str| -> String {
        map.get(&data["label"].as_str().unwrap())
            .unwrap()
            .get(field)
            .unwrap_or_else(|| panic!("Field '{}' not found in variable data!", field))
            .to_string()
    };

    m.insert(String::from("name"), extract("name"));
    m.insert(String::from("type"), extract("type"));
    m.insert(String::from("help"), extract("help"));
    m.insert(String::from("value"), data["value"].as_str().unwrap().to_string());
    m
}

pub fn is_metric(data: &Map<String, serde_json::Value>) -> bool {
    let map = PADM_VARIABLE_MAP.lock().unwrap();
    match map.get(&data["label"].as_str().unwrap()) {
        Some(..) => true,
        None => false,
    }
}
