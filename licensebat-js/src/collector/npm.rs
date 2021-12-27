use crate::{
    collector::{
        common::{retrieve_from_npm, NPM},
        npm_dependency::NpmDependencies,
    },
    retriever::{NpmRetriever, Retriever},
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Dependency, FileCollector,
};
use std::sync::Arc;
use tracing::instrument;

/// NPM dependency collector
#[derive(Debug, Clone)]
pub struct NpmCollector<R: Retriever> {
    retriever: Arc<R>,
}

impl Default for NpmCollector<NpmRetriever> {
    fn default() -> Self {
        let retriever = NpmRetriever::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> NpmCollector<R> {
    pub fn new(retriever: R) -> Self {
        Self {
            retriever: Arc::new(retriever),
        }
    }
}

impl<R: Retriever> Collector for NpmCollector<R> {
    fn get_name(&self) -> String {
        NPM.to_string()
    }
}

impl<R: Retriever> FileCollector for NpmCollector<R> {
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
        retrieve_from_npm(npm_deps.into_iter(), self.retriever.clone())
    }
}
