use crate::{
    collector::common::{retrieve_from_npm, NPM},
    retriever::NpmRetriever,
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Dependency, FileCollector, Retriever,
};
use std::sync::Arc;
use tracing::instrument;

/// Yarn dependency collector
#[derive(Debug)]
pub struct YarnCollector<R: Retriever> {
    retriever: Arc<R>,
}

impl Default for YarnCollector<NpmRetriever> {
    fn default() -> Self {
        let retriever = NpmRetriever::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> YarnCollector<R> {
    pub fn new(retriever: R) -> Self {
        Self {
            retriever: Arc::new(retriever),
        }
    }
}

impl<R: Retriever> Collector for YarnCollector<R> {
    fn get_name(&self) -> String {
        NPM.to_string()
    }
}

impl<R: Retriever> FileCollector for YarnCollector<R> {
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

        retrieve_from_npm(npm_deps, self.retriever.clone())
    }
}
