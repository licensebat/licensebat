use crate::dependency::RetrievedDependency;
use futures::future::{self, BoxFuture};

/// Generic trait for [`Retriever`].
/// Most of the retrievers will use this trait.
/// Some others, may need specific information. For instance, it may be necessary to know the repo url and branch/tag/ref.
/// For those kind of retrievers this trait won't make any sense.
/// So, this trait only exists to avoid duplicating code as it's the most common behavior in retrievers.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// The associated error which can be returned.
    type Error: std::fmt::Debug + std::fmt::Display;

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
        let dep = RetrievedDependency {
            name: dep_name.to_string(),
            version: dep_version.to_string(),
            ..RetrievedDependency::default()
        };
        Box::pin(future::ok(dep))
    }
}
