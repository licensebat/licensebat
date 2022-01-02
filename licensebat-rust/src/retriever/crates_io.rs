#![allow(deprecated)]

use super::utils::crates_io_retrieved_dependency;
use crate::retriever::docs_rs::Retriever as DocsRetriever;
use askalono::Store;
use futures::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use licensebat_core::{Dependency, RetrievedDependency};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

/// Trait used by the [`CratesIo`] struct to retrieve dependencies.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// Future that resolves to a [`RetrievedDependency`].
    /// It cannot fail.
    type Response: Future<Output = RetrievedDependency> + Send;
    /// Validates dependency's information from the original source.
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response;
}

#[deprecated(
    since = "0.0.3",
    note = "Consider using DocsRs retriever instead. We're just keeping this one just in case docs.rs doesn't work."
)]
pub struct CratesIo {
    client: Client,
    store: Arc<Option<Store>>,
}

impl CratesIo {
    /// Creates a new [`CratesIo`] [`Retriever`] using the given [`reqwest::Client`].
    #[must_use]
    pub const fn new(client: Client, store: Arc<Option<Store>>) -> Self {
        Self { client, store }
    }
}

impl Default for CratesIo {
    /// Creates a new [`CratesIo`] [`Retriever`].
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
    type Response = BoxFuture<'static, RetrievedDependency>;

    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response {
        let url = format!(
            "https://crates.io/api/v1/crates/{}/{}",
            dep_name, dep_version
        );

        let dependency = Dependency {
            name: dep_name.to_string(),
            version: dep_version.to_string(),
        };

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
                        docs_rs
                            .get_dependency(&dependency.name, &dependency.version)
                            .await
                    } else {
                        // TODO: ADD SUPPORT FOR MULTIPLE LICENSES by using the spdx crate
                        let licenses = vec![license.to_string()];
                        crates_io_retrieved_dependency(&dependency, Some(licenses), None, None)
                    }
                } else {
                    crates_io_retrieved_dependency(
                        &dependency,
                        None,
                        Some("No license found in Crates.io API"),
                        None,
                    )
                }
            };
            Ok::<_, anyhow::Error>(retrieved_dependency)
        }
        .unwrap_or_else(move |e| {
            let error = e.to_string();
            crates_io_retrieved_dependency(&dep_clone, None, Some(error.as_str()), None)
        })
        .boxed()
    }
}
