use crate::{
    collector::common::{retrieve_from_npm, NPM},
    retriever::{self, npm::Retriever},
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Dependency, FileCollector,
};
use std::sync::Arc;
use tracing::instrument;

/// Yarn dependency collector
#[derive(Debug)]
pub struct Yarn<R: Retriever> {
    retriever: Arc<R>,
}

impl Default for Yarn<retriever::Npm> {
    fn default() -> Self {
        let retriever = retriever::Npm::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> Yarn<R> {
    pub fn new(retriever: R) -> Self {
        Self {
            retriever: Arc::new(retriever),
        }
    }
}

impl<R: Retriever> Collector for Yarn<R> {
    fn get_name(&self) -> String {
        NPM.to_string()
    }
}

impl<R: Retriever> FileCollector for Yarn<R> {
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

        let retriever = self.retriever.clone();
        Ok(retrieve_from_npm(npm_deps, &retriever))
    }
}
