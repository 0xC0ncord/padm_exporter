use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub data: Vec<Variable>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Variable {
    pub attributes: Attributes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attributes {
    pub device_name: String,
    pub label: String,
    pub value: String,
    pub raw_value: String,
    pub enum_values: Vec<EnumValue>,
}
impl Attributes {
    pub fn get_raw(&self) -> Result<f64> {
        match self.label.as_str() {
            "Firmware Version" => Ok(1.0),
            _ => self
                .raw_value
                .parse()
                .map(|v: f64| (v * 10.0).round() / 10.0)
                .map_err(Into::into),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumValue {
    pub name: String,
}
