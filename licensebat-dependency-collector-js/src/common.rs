use futures::FutureExt;
use licensebat_core::{
    dependency_collector::RetrievedDependencyStreamResult, Dependency, DependencyRetriever,
};
use licensebat_dependency_retriever_js_npm::NpmDependencyRetriever;
use tracing::instrument;

pub const NPM: &str = "npm";

#[instrument(skip(deps, client))]
pub fn retrieve_from_npm<'a, I>(
    deps: I,
    client: reqwest::Client,
) -> RetrievedDependencyStreamResult<'a>
where
    I: Iterator<Item = Dependency>,
{
    let retriever = NpmDependencyRetriever::with_client(client);

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
