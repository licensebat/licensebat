//! [`Retriever`] that uses the [crates.io API](https://crates.io).
//!
//! Note that this API is subjected to rate limits and may fail from time to time.
//!
//! Here you can find both the trait and the implementation.
//!
//! Usually, [`Collectors`](licensebat_core::Collector) are generic over a [`Retriever`] (or several). This comes in handy for mocking the [`Retriever`] in our tests.
//!
//! [`Retriever`]: crate::retriever::crates_io::Retriever

#![allow(deprecated)]

use super::utils::crates_io_retrieved_dependency;
use crate::retriever::docs_rs::Retriever as DocsRetriever;
use askalono::Store;
use futures::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use licensebat_core::{Dependency, Dependency};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

/// Trait used by the [`CratesIo`] struct to retrieve dependencies.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// Future that resolves to a [`Dependency`].
    /// It cannot fail.
    type Response: Future<Output = Dependency> + Send;
    /// Validates dependency's information from the original source.
    fn get_dependency(&self, dependency: Dependency) -> Self::Response;
}

/// [`crates.io`] [`Retriever`] implementation.
///
/// It uses [`reqwest::Client`] to fetch the [`crates.io`] API and retrieve the metadata of a dependency..
///
/// Note that this [`Retriever`] implementation is **deprecated** in favour of the [`DocsRs`] one.
///
/// Although this [`Retriever`] is faster than the [`DocsRs`] one for recent uploaded crates (docs.rs takes a while to compile the docs of a crate), it's slower if the crate doesn't declare a license in its metadata (only a license-file) as it will have to make more requests to get it.
///
/// Indeed, it basically ends up calling the [`DocsRs`] [`Retriever`] under the hood when it faces this scenario.
///
/// You can provide yourself an instance of [`reqwest::Client`] by using the [`CratesIo::new`] constructor.
///
/// If you use [`CratesIo::default`], it will instantiate a new [`reqwest::Client`] under the hood.
///
/// [`crates.io`]: https://crates.io
/// [`DocsRs`]: DocsRetriever
#[deprecated(
    since = "0.0.3",
    note = "Consider using DocsRs retriever instead. We're just keeping this one just in case docs.rs doesn't work."
)]
pub struct CratesIo {
    client: Client,
    store: Arc<Option<Store>>,
}

impl CratesIo {
    /// Creates a new [`Retriever`].
    /// If you want to reuse a [`reqwest::Client`] pool consider using the [`CratesIo::new`] method.
    #[must_use]
    pub const fn new(client: Client, store: Arc<Option<Store>>) -> Self {
        Self { client, store }
    }
}

impl Default for CratesIo {
    /// Creates a new [`Retriever`] using the given [`reqwest::Client`].
    /// If you don't want to pass a [`reqwest::Client`] instance, consider using the [`CratesIo::default`] method.
    fn default() -> Self {
        Self::new(Client::new(), Arc::new(None))
    }
}

impl Clone for CratesIo {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            store: self.store.clone(),
        }
    }
}

impl std::fmt::Debug for CratesIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocsRs")
            .field("client", &self.client)
            .field(
                "store",
                if self.store.is_some() {
                    &"Some(Store)"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl Retriever for CratesIo {
    type Response = BoxFuture<'static, Dependency>;

    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dependency: Dependency) -> Self::Response {
        let url = format!(
            "https://crates.io/api/v1/crates/{}/{}",
            dependency.name, dependency.version
        );

        let dep_clone = dependency.clone();
        let client = self.client.clone();
        let store = self.store.clone();

        async move {
            let metadata: Value = client
                .get(&url)
                .header("User-Agent", "licensebat-cli (licensebat.com)")
                .send()
                .await?
                .json()
                .await?;

            let retrieved_dependency = {
                let license = metadata["version"]["license"].clone();
                if let Some(license) = license.as_str() {
                    // this should always be informed.
                    // either by the declared license in the crates' Cargo.toml
                    // or by a generic `non-standard` license.
                    if license == "non-standard" {
                        // we're going to use the docs.rs retriever here
                        let docs_rs = super::docs_rs::DocsRs::new(client, store);
                        docs_rs.get_dependency(dependency).await
                    } else {
                        // TODO: ADD SUPPORT FOR MULTIPLE LICENSES by using the spdx crate
                        let licenses = vec![license.to_string()];
                        crates_io_retrieved_dependency(
                            &dependency,
                            Some(licenses),
                            None,
                            None,
                            None,
                        )
                    }
                } else {
                    crates_io_retrieved_dependency(
                        &dependency,
                        None,
                        Some("No license found in Crates.io API"),
                        None,
                        None,
                    )
                }
            };
            Ok::<_, anyhow::Error>(retrieved_dependency)
        }
        .unwrap_or_else(move |e| {
            let error = e.to_string();
            crates_io_retrieved_dependency(&dep_clone, None, Some(error.as_str()), None, None)
        })
        .boxed()
    }
}
