use crate::{
    collector::{
        common::{retrieve_from_npm, NPM},
        npm_dependency::NpmDependencies,
    },
    retriever::{self, npm::Retriever},
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Dependency, FileCollector,
};
use std::sync::Arc;
use tracing::instrument;

/// NPM dependency [`FileCollector`] generic over [`Retriever`].
///
/// This [`FileCollector`] parses a `package-lock.json` file and then retrieves information about the dependencies from the npm registry API.
#[derive(Debug, Clone)]
pub struct Npm<R: Retriever> {
    retriever: Arc<R>,
}

impl Default for Npm<retriever::Npm> {
    /// Creates a new [`Npm`] [`FileCollector`] that uses a [`retriever::Npm`].
    fn default() -> Self {
        let retriever = retriever::Npm::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> Npm<R> {
    /// Creates a new [`Npm`] [`FileCollector`].
    pub fn new(retriever: R) -> Self {
        Self {
            retriever: Arc::new(retriever),
        }
    }
}

impl<R: Retriever> Collector for Npm<R> {
    fn get_name(&self) -> String {
        NPM.to_string()
    }
}

impl<R: Retriever> FileCollector for Npm<R> {
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

        Ok(retrieve_from_npm(npm_deps, &self.retriever))
    }
}
