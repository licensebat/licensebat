use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependency {
    pub version: String,
    #[serde(rename = "dev")]
    pub is_dev: Option<bool>,
    #[serde(rename = "optional")]
    pub is_optional: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependencies {
    pub dependencies: HashMap<String, NpmDependency>,
}
