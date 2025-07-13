use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub data: Vec<Variable>,
}
impl ApiResponse {
    pub fn new() -> ApiResponse {
        ApiResponse { data: Vec::new() }
    }
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
}
