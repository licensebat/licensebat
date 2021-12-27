use futures::FutureExt;
use licensebat_core::{collector::RetrievedDependencyStreamResult, Dependency, Retriever};
use std::sync::Arc;
use tracing::instrument;

pub const NPM: &str = "npm";

#[instrument(skip(deps, retriever))]
pub fn retrieve_from_npm<'a, I, R>(
    deps: I,
    retriever: Arc<R>,
) -> RetrievedDependencyStreamResult<'a>
where
    I: Iterator<Item = Dependency>,
    R: Retriever + 'a,
{
    Ok(deps
        .into_iter()
        .map(|dep| {
            retriever
                .get_dependency(&dep.name, &dep.version)
                .map(std::result::Result::unwrap) // TODO: handle error
                .boxed()
        })
        .collect())
}
