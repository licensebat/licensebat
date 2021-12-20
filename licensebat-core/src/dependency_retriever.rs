use crate::dependencies::RetrievedDependency;

/// Trait to be implemented by every [`DependencyRetriever`].
/// It will get the information needed about a specific dependency.
pub trait DependencyRetriever {
    /// The associated error which can be returned.
    type Error;

    /// Future that resolves to a [`RetrievedDependency`].
    type Future: futures::Future<Output = Result<RetrievedDependency, Self::Error>>;

    /// Validates dependency's information from the original source.
    /// i.e. For JS, it will go to the npm registry and look for the license in there.
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Future;
}
