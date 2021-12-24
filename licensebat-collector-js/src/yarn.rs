use crate::common::{retrieve_from_npm, NPM};
use licensebat_core::{collector::RetrievedDependencyStreamResult, Collector, Dependency};
use reqwest::Client;
use tracing::instrument;

/// Yarn dependency collector
#[derive(Debug)]
pub struct Yarn(Client);

impl Default for Yarn {
    fn default() -> Self {
        Self::with_client(Client::new())
    }
}

impl Yarn {
    pub fn with_client(client: Client) -> Self {
        Self(client)
    }
}

impl Collector for Yarn {
    fn get_name(&self) -> String {
        NPM.to_string()
    }

    fn get_dependency_filename(&self) -> String {
        String::from("yarn.lock")
    }

    #[instrument(skip(self))]
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult {
        let npm_deps = yarn_lock_parser::parse_str(dependency_file_content)?
            .into_iter()
            .map(|entry| Dependency {
                name: entry.name.to_owned(),
                version: entry.version.to_owned(),
            });

        retrieve_from_npm(npm_deps, self.0.clone())
    }
}
