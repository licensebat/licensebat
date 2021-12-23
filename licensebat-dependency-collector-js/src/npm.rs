use crate::common::{retrieve_from_npm, NPM};
use crate::npm_dependency::NpmDependencies;
use licensebat_core::{
    dependency_collector::RetrievedDependencyStreamResult, Dependency, DependencyCollector,
};
use reqwest::Client;
use tracing::instrument;

/// NPM dependency collector
#[derive(Debug)]
pub struct Npm(Client);

impl Default for Npm {
    fn default() -> Self {
        Self::with_client(Client::new())
    }
}

impl Npm {
    pub fn with_client(client: Client) -> Self {
        Self(client)
    }
}

impl DependencyCollector for Npm {
    fn get_name(&self) -> String {
        NPM.to_string()
    }

    fn get_dependency_filename(&self) -> String {
        String::from("package-lock.json")
    }

    #[instrument(skip(self))]
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult {
        let npm_deps = serde_json::from_str::<NpmDependencies>(dependency_file_content)?
            .dependencies
            .into_iter()
            .map(|(key, value)| Dependency {
                // TODO: for yarn, this key includes the version (as there can be more than one version of a package declared)
                name: key,
                version: value.version,
            });
        retrieve_from_npm(npm_deps.into_iter(), self.0.clone())
    }
}
