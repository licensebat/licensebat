use crate::{
    collector::common::{retrieve_from_npm, NPM},
    retriever::{self, npm::Retriever},
};
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, licrc::LicRc, Collector, Dependency, FileCollector,
};
use tracing::instrument;

/// Yarn dependency [`FileCollector`] generic over [`Retriever`].
///
/// This [`FileCollector`] parses a `yarn.lock` file and then retrieves information about the dependencies from the npm registry API.
#[derive(Debug)]
pub struct Yarn<R: Retriever> {
    retriever: R,
}

impl Default for Yarn<retriever::Npm> {
    fn default() -> Self {
        let retriever = retriever::Npm::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> Yarn<R> {
    /// Creates a new [`Yarn`] [`FileCollector`].
    pub const fn new(retriever: R) -> Self {
        Self { retriever }
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
    fn get_dependencies(
        &self,
        dependency_file_content: &str,
        licrc: &LicRc,
    ) -> RetrievedDependencyStreamResult {
        let npm_deps = yarn_lock_parser::parse_str(dependency_file_content)?
            .into_iter()
            .map(|entry| Dependency {
                name: entry.name.to_owned(),
                version: entry.version.to_owned(),
                is_dev: None,
                is_optional: None,
            })
            .filter(|dep| licrc.filter_dependencies_before_retrieval(dep));

        Ok(retrieve_from_npm(npm_deps, &self.retriever))
    }
}
