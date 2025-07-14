use anyhow::{Context, Result};
use prometheus::{GaugeVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MetricKey {
    pub name: String,
    pub help: String,
    pub labels: Vec<String>,
    pub is_enum: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PADMMetric {
    FirmwareVersion,
    OperatingMode,
    LcdDisplayUnits,
    DehumidifyingMode,
    WaterFault,
    AutomaticFanSpeedState,
    FanSpeed,
    FanAlwaysOn,
    QuietMode,
    SetPointTemperature,
    RemoteTemperatureSensorState,
    ReturnAirTemperature,
    RemoteSetPointTemperature,
    Temperature,
    TemperatureSupported,
    HumiditySupported,
    ContactInputCount,
    ContactOutputCount,
    TempLowCritThreshold,
    TempHighCritThreshold,
}
impl PADMMetric {
    pub fn to_metric_key(&self) -> MetricKey {
        let (name, help, labels, is_enum) = match self {
            PADMMetric::FirmwareVersion => (
                "firmware_version",
                "Device firmware version.",
                vec!["target".into(), "device".into(), "version".into()],
                false,
            ),
            PADMMetric::OperatingMode => (
                "operating_mode",
                "Device operating mode.",
                vec!["target".into(), "device".into(), "mode".into()],
                true,
            ),
            PADMMetric::LcdDisplayUnits => (
                "lcd_display_units",
                "Units displayed on the LCD screen.",
                vec!["target".into(), "device".into(), "mode".into()],
                true,
            ),
            PADMMetric::DehumidifyingMode => (
                "dehumidifying_mode",
                "Device dehumidifying mode.",
                vec!["target".into(), "device".into(), "mode".into()],
                true,
            ),
            PADMMetric::WaterFault => (
                "water_fault",
                "Device water fault status.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::AutomaticFanSpeedState => (
                "automatic_fan_speed_state",
                "Device automatic fan speed status.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::FanSpeed => (
                "fan_speed",
                "Device fan speed setting.",
                vec!["target".into(), "device".into(), "mode".into()],
                true,
            ),
            PADMMetric::FanAlwaysOn => (
                "fan_always_on",
                "Device fan always on setting.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::QuietMode => (
                "quiet_mode",
                "Device quiet mode setting.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::SetPointTemperature => (
                "set_point_temperature",
                "Set point temperature in celsius.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::RemoteTemperatureSensorState => (
                "remote_temperature_sensor_state",
                "Remote temperature sensor state.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::ReturnAirTemperature => (
                "return_air_temperature",
                "Temperature of the return air in celsius.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::RemoteSetPointTemperature => (
                "remote_set_point_temperature",
                "Set point temperature of the remote sensor.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::Temperature => (
                "temperature",
                "Current detected temperature.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::TemperatureSupported => (
                "temperature_supported",
                "Whether the device supports temperature sensing.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::HumiditySupported => (
                "humidity_supported",
                "Whether the device supports humidity sensing.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::ContactInputCount => (
                "contact_input_count",
                "GPIO input contact counter.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::ContactOutputCount => (
                "contact_output_count",
                "GPIO output contact counter.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::TempLowCritThreshold => (
                "temp_low_crit_threshold",
                "Critically low temperature.",
                vec!["target".into(), "device".into()],
                false,
            ),
            PADMMetric::TempHighCritThreshold => (
                "temp_high_crit_threshold",
                "Critically high temperature.",
                vec!["target".into(), "device".into()],
                false,
            ),
        };

        MetricKey {
            name: "padm_".to_string() + name,
            help: help.to_string(),
            labels,
            is_enum,
        }
    }
    pub fn from_label(label: &str) -> Option<Self> {
        use PADMMetric::*;
        Some(match label {
            "Firmware Version" => FirmwareVersion,
            "Operating Mode" => OperatingMode,
            "LCD Display Units (Cooling)" => LcdDisplayUnits,
            "Dehumidifying Mode" => DehumidifyingMode,
            "Water Fault" => WaterFault,
            "Fan Speed - Auto" => AutomaticFanSpeedState,
            "Fan Speed" => FanSpeed,
            "Fan Always On" => FanAlwaysOn,
            "Quiet Mode" => QuietMode,
            "Set Point Temperature (C)" => SetPointTemperature,
            "Remote Temperature Sensor" => RemoteTemperatureSensorState,
            "Return Air Temperature (C)" => ReturnAirTemperature,
            "Remote Set Point Temperature (C)" => RemoteSetPointTemperature,
            "Temperature (C)" => Temperature,
            "Temperature Supported" => TemperatureSupported,
            "Humidity Supported" => HumiditySupported,
            "Contact Input Count" => ContactInputCount,
            "Contact Output Count" => ContactOutputCount,
            "Temperature (C) Low Critical Threshold" => TempLowCritThreshold,
            "Temperature (C) High Critical Threshold" => TempHighCritThreshold,
            _ => return None,
        })
    }
}

pub struct MetricsRegistry {
    pub metrics: Mutex<HashMap<MetricKey, GaugeVec>>,
    pub registry: Registry,
}
impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(HashMap::new()),
            registry: Registry::new(),
        }
    }
    pub fn update_metric(&self, key: &MetricKey, label_values: &[&str], value: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().unwrap();
        if !metrics.contains_key(key) {
            let label_refs: Vec<&str> = key.labels.iter().map(String::as_str).collect();

            GaugeVec::new(Opts::new(&key.name, &key.help), &label_refs)
                .and_then(|g| {
                    self.registry.register(Box::new(g.clone()))?;
                    metrics.insert(key.clone(), g);
                    Ok(())
                })
                .context(format!("Failed to register metric {}", &key.name))?;
        }

        if let Some(g) = metrics.get(key) {
            g.with_label_values(label_values).set(value);
        }
        Ok(())
    }
}
