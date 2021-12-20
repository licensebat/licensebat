use licensebat_core::Dependency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependency {
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NpmDependencies {
    dependencies: HashMap<String, NpmDependency>,
}

impl NpmDependencies {
    /// Collects all the dependencies into a a vector of [Dependency].
    /// It consumes the original `HashMap`.
    pub fn into_vec_collection(self) -> Vec<Dependency> {
        self.dependencies
            .into_iter()
            .map(|(key, value)| Dependency {
                name: key, // TODO: for yarn, this key includes de version (as there can be more than one version of a package declared)
                version: value.version,
            })
            .collect()
    }
}
