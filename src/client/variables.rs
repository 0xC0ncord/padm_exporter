use log::error;
use once_cell::sync::Lazy;
use serde_json::Map;
use std::{collections::HashMap, sync::Mutex};

static PADM_VARIABLE_MAP: Lazy<Mutex<HashMap<&str, HashMap<&str, &str>>>> = Lazy::new(|| {
    let map = HashMap::from([
        (
            "Firmware Version",
            HashMap::from([
                ("name", "firmware_version"),
                ("type", "gauge"),
                ("help", "Device firmware version."),
            ]),
        ),
        (
            "Operating Mode",
            HashMap::from([
                ("name", "operating_mode"),
                ("type", "gauge"),
                ("help", "Device operating mode."),
            ]),
        ),
        (
            "LCD Display Units (Cooling)",
            HashMap::from([
                ("name", "lcd_display_units"),
                ("type", "gauge"),
                ("help", "Units displayed on the LCD screen."),
            ]),
        ),
        (
            "Dehumidifying Mode",
            HashMap::from([
                ("name", "dehumidifying_mode"),
                ("type", "gauge"),
                ("help", "Device dehumidifying mode."),
            ]),
        ),
        (
            "Water Fault",
            HashMap::from([
                ("name", "water_fault"),
                ("type", "gauge"),
                ("help", "Device water fault status."),
            ]),
        ),
        (
            "Fan Speed - Auto",
            HashMap::from([
                ("name", "automatic_fan_speed_state"),
                ("type", "gauge"),
                ("help", "Device automatic fan speed status."),
            ]),
        ),
        (
            "Fan Speed",
            HashMap::from([
                ("name", "fan_speed"),
                ("type", "gauge"),
                ("help", "Device fan speed setting."),
            ]),
        ),
        (
            "Fan Always On",
            HashMap::from([
                ("name", "fan_always_on"),
                ("type", "gauge"),
                ("help", "Device fan always on setting."),
            ]),
        ),
        (
            "Quiet Mode",
            HashMap::from([
                ("name", "quiet_mode"),
                ("type", "gauge"),
                ("help", "Device quiet mode setting."),
            ]),
        ),
        (
            "Set Point Temperature (C)",
            HashMap::from([
                ("name", "set_point_temperature"),
                ("type", "gauge"),
                ("help", "Set point temperature in celsius."),
            ]),
        ),
        (
            "Remote Temperature Sensor",
            HashMap::from([
                ("name", "remote_temperature_sensor_state"),
                ("type", "gauge"),
                ("help", "Remote temperature sensor state."),
            ]),
        ),
        (
            "Return Air Temperature (C)",
            HashMap::from([
                ("name", "return_air_temperature"),
                ("type", "gauge"),
                ("help", "Temperature of the return air in celsius."),
            ]),
        ),
        (
            "Remote Set Point Temperature (C)",
            HashMap::from([
                ("name", "remote_set_point_temperature"),
                ("type", "gauge"),
                ("help", "Set point temperature of the remote sensor."),
            ]),
        ),
        (
            "Temperature (C)",
            HashMap::from([
                ("name", "temperature"),
                ("type", "gauge"),
                ("help", "Current detected temperature."),
            ]),
        ),
        (
            "Temperature Supported",
            HashMap::from([
                ("name", "temperature_supported"),
                ("type", "gauge"),
                ("help", "Whether the device supports temperature sensing."),
            ]),
        ),
        (
            "Humidity Supported",
            HashMap::from([
                ("name", "humidity_supported"),
                ("type", "gauge"),
                ("help", "Whether the device supports humidity sensing."),
            ]),
        ),
        (
            "Contact Input Count",
            HashMap::from([
                ("name", "contact_input_count"),
                ("type", "counter"),
                ("help", "GPIO input contact counter."),
            ]),
        ),
        (
            "Contact Output Count",
            HashMap::from([
                ("name", "contact_output_count"),
                ("type", "counter"),
                ("help", "GPIO output contact counter."),
            ]),
        ),
        (
            "Temperature (C) Low Critical Threshold",
            HashMap::from([
                ("name", "temp_low_crit_threshold"),
                ("type", "gauge"),
                ("help", "Critically low temperature."),
            ]),
        ),
        (
            "Temperature (C) High Critical Threshold",
            HashMap::from([
                ("name", "temp_high_crit_threshold"),
                ("type", "gauge"),
                ("help", "Critically high temperature."),
            ]),
        ),
    ]);
    Mutex::new(map)
});

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    vtype: String,
    help: String,
    value: String,
    labels: Option<HashMap<String, String>>,
}
impl Variable {
    pub fn get(&self, field: &str) -> &str {
        match field {
            "name" => &self.name,
            "type" => &self.vtype,
            "help" => &self.help,
            "value" => &self.value,
            _ => "",
        }
    }
    pub fn labels(&self) -> &Option<HashMap<String, String>> {
        &self.labels
    }
}

/// Mutate the variable's value if needed
pub fn mutate_variable<'a>(
    name: &'a str,
    value: &'a str
) -> (&'a str, Option<HashMap<String, String>>) {
    match name {
        "firmware_version" => ("1", Some(HashMap::from([(String::from("version"), value.to_string())]))),
        "operating_mode" => (
            match value {
                "Off" => "0",
                "Idle" => "1",
                "Cooling" => "2",
                // 3 skipped intentionally
                "Dehumidifying" => "4",
                "Defrosting" => "5",
                "Not Connected" => "6",
                _ => "6",
            },
            Some(HashMap::from([(String::from("mode"), value.to_string())]))
        ),
        "lcd_display_units" => (
            match value {
                "Metric" => "0",
                "English" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("units"), value.to_string())]))
        ),
        "dehumidifying_mode" => (
            match value {
                "Off" => "0",
                "On" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("state"), value.to_string())]))
        ),
        "water_fault" => (
            match value {
                "Not Full" => "0",
                "Full" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("fault"), value.to_string())]))
        ),
        "automatic_fan_speed_state" => (
            match value {
                "Off" => "0",
                "On" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("state"), value.to_string())]))
        ),
        "fan_speed" => (
            match value {
                "Low" => "1",
                "Medium" => "3",
                "High" => "5",
                "Auto" => "6",
                "Off" => "10",
                "Low (Auto)" => "11",
                "Medium Low (Auto)" => "12",
                "Medium (Auto)" => "13",
                "Medium High (Auto)" => "14",
                "High (Auto)" => "15",
                _ => "6",
            },
            Some(HashMap::from([(String::from("speed"), value.to_string())]))
        ),
        "fan_always_on" => (
            match value {
                "No" => "0",
                "Yes" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("enabled"), value.to_string())]))
        ),
        "quiet_mode" => (
            match value {
                "Disabled" => "0",
                "Enabled" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("state"), value.to_string())]))
        ),
        "remote_temperature_sensor_state" => (
            match value {
                "Disabled" => "0",
                "Enabled" => "1",
                _ => "1",
            },
            Some(HashMap::from([(String::from("state"), value.to_string())]))
        ),
        "temperature_supported" => (
            match value {
                "Yes" => "1",
                "No" => "2",
                _ => "1",
            },
            Some(HashMap::from([(String::from("supported"), value.to_string())]))
        ),
        "humidity_supported" => (
            match value {
                "Yes" => "1",
                "No" => "2",
                _ => "1",
            },
            Some(HashMap::from([(String::from("supported"), value.to_string())]))
        ),
        _ => (value, None)
    }
}

pub fn unpack_variable(data: &Map<String, serde_json::Value>) -> Variable {
    let map = PADM_VARIABLE_MAP.lock().unwrap();
    let extract = |field: &str| -> String {
        // Get the map containing the label
        let var = match map.get(&data["label"].as_str().unwrap()) {
            Some(s) => s,
            None => {
                error!("Key 'label' not found in devices data!");
                return String::new();
            }
        };

        match var.get(field) {
            Some(s) => s.to_string(),
            None => {
                error!("Field '{field}' not found in variable data!");
                String::new()
            }
        }
    };

    let var_name = extract("name");

    let (value, labels) = mutate_variable(&var_name, data["value"].as_str().unwrap());

    Variable {
        name: var_name.to_owned(),
        vtype: extract("type"),
        help: extract("help"),
        value: value.to_owned(),
        labels,
    }
}

pub fn is_metric(data: &Map<String, serde_json::Value>) -> bool {
    let map = PADM_VARIABLE_MAP.lock().unwrap();
    match map.get(&data["label"].as_str().unwrap()) {
        Some(..) => true,
        None => false,
    }
}
