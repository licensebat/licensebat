use crate::retriever::npm::Retriever;
use futures::FutureExt;
use licensebat_core::{collector::RetrievedDependencyStream, Dependency};
use tracing::instrument;

/// String used to identify the type of dependency
pub const NPM: &str = "npm";

#[instrument(skip(deps, retriever))]
pub fn retrieve_from_npm<'a, I, R>(deps: I, retriever: &R) -> RetrievedDependencyStream<'a>
where
    I: Iterator<Item = Dependency>,
    R: Retriever + 'a,
{
    let iter = deps
        .into_iter()
        .map(|dep| retriever.get_dependency(dep).boxed())
        .collect();

    RetrievedDependencyStream::new(iter)
}
