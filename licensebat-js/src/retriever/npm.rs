//! [`Retriever`] that uses the [npm API](https://registry.npmjs.org/).
//!
//! Note that this API is subjected to rate limits and may fail from time to time.
//!
//! Here you can find both the trait and the implementation.
//!
//! Usually, [`Collectors`](licensebat_core::Collector) are generic over a [`Retriever`] (or several). This comes in handy for mocking the [`Retriever`] in our tests.
//!
//! [`Retriever`]: crate::retriever::npm::Retriever
use crate::retriever::npm_metadata::NpmMetadata;
use futures::{
    future::{self, BoxFuture},
    Future, FutureExt, TryFutureExt,
};
use licensebat_core::{Dependency, RetrievedDependency};
use reqwest::Client;
use serde_json::Value;
use tracing::instrument;

/// Trait implemented by the [`Npm`] struct to retrieve dependencies.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// Future that resolves to a [`RetrievedDependency`].
    /// It cannot fail.
    /// If there's some error while retrieving the dependency, it will return the error in the [`RetrievedDependency`]'s `error` field.
    type Response: Future<Output = RetrievedDependency> + Send;
    /// Validates dependency's information from the original source.
    fn get_dependency(&self, dependency: Dependency) -> Self::Response;
}

/// Npm [`Retriever`] implementation.
///
/// It uses [`reqwest::Client`] to connect to the npm registry and retrieve the metadata of a dependency.
///
/// You can provide yourself an instance of [`reqwest::Client`] by using the [`Npm::new`] constructor.
///
/// If you use [`Npm::default`], it will instantiate a new [`reqwest::Client`] under the hood.
#[derive(Debug, Clone)]
pub struct Npm {
    client: Client,
}

impl Default for Npm {
    /// Creates a new [`Retriever`].
    /// If you want to reuse a [`reqwest::Client`] pool consider using the [`Npm::new`] method.
    fn default() -> Self {
        Self::new(Client::new())
    }
}

impl Npm {
    /// Creates a new [`Retriever`] using the given [`reqwest::Client`].
    /// If you don't want to pass a [`reqwest::Client`] instance, consider using the [`Npm::default`] method.
    #[must_use]
    pub const fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Retriever for Npm {
    type Response = BoxFuture<'static, RetrievedDependency>;

    /// Gets a dependency from the [`npm API`](https://registry.npmjs.org/).
    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dependency: Dependency) -> Self::Response {
        let url = format!("https://registry.npmjs.org/{}", dependency.name);

        let dep_clone = dependency.clone();
        let dependency_version = dependency.version.to_string();

        self.client
            .get(&url)
            .send()
            .and_then(reqwest::Response::json)
            .map_ok(|metadata: Value| {
                // get general license
                let license = metadata["license"].clone();
                // get info from specific version
                let version = metadata["versions"][dependency_version].clone();
                serde_json::from_value::<NpmMetadata>(version)
                    .ok()
                    .and_then(|mut md| {
                        if md.license.is_none() {
                            // use generic if no license is found in the version
                            md.license = match license {
                                Value::String(lic) => Some(lic),
                                Value::Object(lic) => lic
                                    .get("type")
                                    .and_then(serde_json::Value::as_str)
                                    .map(std::borrow::ToOwned::to_owned),
                                _ => None,
                            }
                        }
                        md.get_licenses()
                    })
            })
            .map_ok(move |licenses: Option<Vec<String>>| {
                retrieved_dependency(&dep_clone, licenses, None)
            })
            .or_else(move |e| future::ok(retrieved_dependency(&dependency, None, Some(e))))
            .map(std::result::Result::<RetrievedDependency, std::convert::Infallible>::unwrap)
            .boxed()
    }
}

fn retrieved_dependency(
    dependency: &Dependency,
    licenses: Option<Vec<String>>,
    error: Option<reqwest::Error>,
) -> RetrievedDependency {
    let url = format!(
        "https://www.npmjs.com/package/{}/v/{}",
        dependency.name, dependency.version
    );

    RetrievedDependency::new(
        dependency.name.clone(),
        dependency.version.clone(),
        crate::NPM.to_owned(),
        Some(url),
        licenses,
        error.map(|e| e.to_string()),
        None,
        None,
        dependency.is_dev,
        dependency.is_optional,
    )
}
