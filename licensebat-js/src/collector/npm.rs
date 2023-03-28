use crate::{
    collector::common::{retrieve_from_npm, NPM},
    retriever::{self, npm::Retriever},
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Dependency, FileCollector,
};
use tracing::instrument;

/// NPM dependency [`FileCollector`] generic over [`Retriever`].
///
/// This [`FileCollector`] parses a `package-lock.json` file and then retrieves information about the dependencies from the npm registry API.
#[derive(Debug, Clone)]
pub struct Npm<R: Retriever> {
    retriever: R,
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
    pub const fn new(retriever: R) -> Self {
        Self { retriever }
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
        let npm_deps = package_lock_json_parser::parse_dependencies(dependency_file_content)?
            .into_iter()
            .map(|dep| Dependency {
                // TODO: for yarn, this key includes the version (as there can be more than one version of a package declared)
                name: dep.name,
                version: dep.version,
                is_dev: Some(dep.is_dev),
                is_optional: Some(dep.is_optional),
            });

        Ok(retrieve_from_npm(npm_deps, &self.retriever))
    }
}
