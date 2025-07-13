use crate::client::variables::Variable;

#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub variables: Vec<Variable>,
}
