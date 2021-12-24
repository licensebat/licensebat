use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependency {
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependencies {
    pub dependencies: HashMap<String, NpmDependency>,
}
