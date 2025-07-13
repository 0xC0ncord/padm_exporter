use crate::client::variables::Variable;

#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub device_type: String,
    pub variables: Vec<Variable>,
}
