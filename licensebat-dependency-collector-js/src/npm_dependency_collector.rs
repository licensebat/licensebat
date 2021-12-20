use crate::npm_dependency::NpmDependencies;
use futures::prelude::*;
use licensebat_core::{
    dependency_collector::RetrievedDependencyStreamResult, DependencyCollector, DependencyRetriever,
};
use licensebat_dependency_retriever_js_npm::NpmDependencyRetriever;
use reqwest::Client;
use tracing::instrument;

const NPM: &str = "npm";

/// NPM navigator
#[derive(Debug)]
pub struct NpmDependencyCollector(pub Client);

impl DependencyCollector for NpmDependencyCollector {
    fn get_name(&self) -> String {
        NPM.to_string()
    }

    fn get_dependency_filename(&self) -> String {
        String::from("package-lock.json")
    }

    #[instrument(skip(self))]
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult {
        let npm_deps =
            serde_json::from_str::<NpmDependencies>(dependency_file_content)?.into_vec_collection();

        let retriever = NpmDependencyRetriever::with_client(self.0.clone());

        Ok(npm_deps
            .into_iter()
            .map(|dep| {
                retriever
                    .get_dependency(&dep.name, &dep.version)
                    .map(std::result::Result::unwrap) // TODO: handle error
                    .boxed()
            })
            .collect())
    }
}
