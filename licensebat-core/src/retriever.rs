use crate::dependency::RetrievedDependency;
use futures::future::{self, BoxFuture};

// TODO: THINK OF THIS TRAIT. Not sure if it makes sense to have it shared amongst retrievers as some of them may have other signatures or needs.
// imagine needing more information than just the name and version. For example, in a git repo, we're going to need the repo url and the branch/tag/ref aside from other information.

/// Trait to be implemented by every [`Retriever`].
/// It will get the information needed about a specific dependency.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// The associated error which can be returned.
    type Error: std::fmt::Debug;

    /// Future that resolves to a [`RetrievedDependency`].
    type Future: futures::Future<Output = Result<RetrievedDependency, Self::Error>> + Send;

    /// Validates dependency's information from the original source.
    /// i.e. For JS, it will go to the npm registry and look for the license in there.
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Future;
}

#[derive(Debug, Default)]
pub struct EmptyRetriever;

impl Retriever for EmptyRetriever {
    type Error = std::convert::Infallible;
    type Future = BoxFuture<'static, Result<RetrievedDependency, Self::Error>>;

    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Future {
        let mut dep = RetrievedDependency::default();
        dep.name = dep_name.to_string();
        dep.version = dep_version.to_string();
        Box::pin(future::ok(dep))
    }
}
