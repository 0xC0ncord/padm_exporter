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
    CoolingMode,
    FanSpeedSetting,
    FaultConditions,

    DeviceUp,
}
impl PADMMetric {
    pub fn to_metric_key(&self) -> MetricKey {
        let (name, help, labels, is_enum) = match self {
            PADMMetric::FirmwareVersion => (
                "firmware_version_info",
                "Firmware version of the device.",
                vec!["device".into(), "version".into()],
                false,
            ),
            PADMMetric::OperatingMode => (
                "operating_mode",
                "Current mode of operation of the device.",
                vec!["device".into(), "mode".into()],
                true,
            ),
            PADMMetric::LcdDisplayUnits => (
                "lcd_display_units",
                "Units displayed on the LCD screen of the device.",
                vec!["device".into(), "mode".into()],
                true,
            ),
            PADMMetric::DehumidifyingMode => (
                "dehumidifier_enabled",
                "Whether the dehumidifier is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::WaterFault => (
                "water_fault_active",
                "Whether a water fault is present.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::AutomaticFanSpeedState => (
                "automatic_fan_speed_enabled",
                "Whether automatic fan speed is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::FanSpeed => (
                "fan_speed",
                "The current fan speed of the device.",
                vec!["device".into(), "mode".into()],
                true,
            ),
            PADMMetric::FanAlwaysOn => (
                "fan_always_on_enabled",
                "Whether the fan always on setting is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::QuietMode => (
                "quiet_mode_enabled",
                "Whether quiet mode is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::SetPointTemperature => (
                "set_point_temperature_celsius",
                "Set point temperature of the device, in degrees Celsius.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::RemoteTemperatureSensorState => (
                "remote_temperature_sensor_enabled",
                "Whether a remote temperature sensor is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::ReturnAirTemperature => (
                "return_air_temperature_celsius",
                "Return air temperature currently reported by the device, in degrees Celsius.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::RemoteSetPointTemperature => (
                "remote_set_point_temperature_celsius",
                "Set point temperature of the remote sensor of the device, in degrees Celsius.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::Temperature => (
                "temperature_celsius",
                "Ambient temperature currently reported by the device, in degrees Celsius.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::TemperatureSupported => (
                "temperature_supported",
                "Whether the device supports temperature sensing.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::HumiditySupported => (
                "humidity_supported",
                "Whether the device supports humidity sensing.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::ContactInputCount => (
                "contact_input_count",
                "GPIO input contact counter of the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::ContactOutputCount => (
                "contact_output_count",
                "GPIO output contact counter of the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::TempLowCritThreshold => (
                "temp_low_crit_threshold",
                "Critically low temperature threshold configured on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::TempHighCritThreshold => (
                "temp_high_crit_threshold",
                "Critically high temperature threshold configured on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::CoolingMode => (
                "cooling_enabled",
                "Whether the cooling mode is enabled on the device.",
                vec!["device".into()],
                false,
            ),
            PADMMetric::FanSpeedSetting => (
                "fan_speed_setting",
                "The current fan speed setting of the device.",
                vec!["device".into(), "mode".into()],
                true,
            ),
            PADMMetric::FaultConditions => (
                "fault_conditions",
                "Whether the device is reporting any faults.",
                vec!["device".into()],
                false,
            ),

            PADMMetric::DeviceUp => (
                "device_up",
                "Whether the device is detected and enabled.",
                vec!["device".into()],
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
            // Known PADM metrics.
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
            "Cooling Mode" => CoolingMode,
            "Fan Speed Setting" => FanSpeedSetting,
            "Fault Conditions" => FaultConditions,

            // Internal metrics.
            "Device Up" => DeviceUp,

            // Everything else.
            _ => return None,
        })
    }
    pub fn is_ignored(label: &str) -> bool {
        matches!(
            label,
            "LCD Display Details"
                | "Hardware Configuration"
                | "Temperature (C) Thresholds and Bounds"
        )
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
