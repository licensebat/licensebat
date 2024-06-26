//! [`Retriever`] that uses the [Docs.rs website].
//!
//! Here you can find both the trait and the implementation.
//!
//! Usually, [`Collectors`](licensebat_core::Collector) are generic over a [`Retriever`] (or several). This comes in handy for mocking the [`Retriever`] in our tests.
//!
//! [`Retriever`]: crate::retriever::docs_rs::Retriever
//! [Docs.rs website]: https://docs.rs/

use super::utils::crates_io_retrieved_dependency;
use askalono::{Store, TextData};
use futures::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use licensebat_core::{Dependency, RetrievedDependency};
use reqwest::Client;
use std::{string::String, sync::Arc};
use thiserror::Error;
use tracing::instrument;

/// Trait used by the [`DocsRs`] struct to retrieve dependencies.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// Future that resolves to a [`RetrievedDependency`].
    /// It cannot fail.
    type Response: Future<Output = RetrievedDependency> + Send;
    /// Validates dependency's information from the original source.
    fn get_dependency(&self, dependency: Dependency) -> Self::Response;
}

/// [`docs.rs`] [`Retriever`] implementation.
///
/// It uses [`reqwest::Client`] to scrap the [`docs.rs`] website and retrieve the metadata of a dependency.
///
/// Note that when a crate is published it takes a while for the [`docs.rs`] website to compile it, so it can take a while to retrieve the metadata of recently uploaded crate.
///
/// You can provide yourself an instance of [`reqwest::Client`] by using the [`DocsRs::new`] constructor.
///
/// If you use [`DocsRs::default`], it will instantiate a new [`reqwest::Client`] under the hood.
///
/// [`docs.rs`]: https://docs.rs
pub struct DocsRs {
    client: Client,
    store: Arc<Option<Store>>,
}

impl DocsRs {
    /// Creates a new [`Retriever`].
    /// If you want to reuse a [`reqwest::Client`] pool consider using the [`DocsRs::new`] method.
    #[must_use]
    pub const fn new(client: Client, store: Arc<Option<Store>>) -> Self {
        Self { client, store }
    }
}

impl Default for DocsRs {
    /// Creates a new [`Retriever`] using the given [`reqwest::Client`].
    /// If you don't want to pass a [`reqwest::Client`] instance, consider using the [`DocsRs::default`] method.
    fn default() -> Self {
        Self::new(Client::new(), Arc::new(None))
    }
}

impl Clone for DocsRs {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            store: self.store.clone(),
        }
    }
}

impl std::fmt::Debug for DocsRs {
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

impl Retriever for DocsRs {
    type Response = BoxFuture<'static, RetrievedDependency>;

    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dependency: Dependency) -> Self::Response {
        let crate_url = docs_rs_url(&dependency.name, &dependency.version);
        let cargo_toml_url = format!("{crate_url}Cargo.toml");

        let dep_clone = dependency.clone();
        let client = self.client.clone();
        let store = self.store.clone();

        async move {
            let html = client
                .get(&cargo_toml_url)
                .header("User-Agent", "licensebat-cli (licensebat.com)")
                .send()
                .await?
                .text()
                .await?;

            // Pattern to get license information from the Cargo.toml
            // docs.rs exposes the content of the Cargo.toml as <code></code>.
            // Unfortunately, the code is mixed with html code that is generated by the docs.rs website.
            // We will clean it and will get the license information.
            let license_info = easy_scraper::Pattern::new(
                r#"<div id="source-code"><pre><code>{{value}}</code></pre></div>"#,
            )
            .map(|pattern| pattern.matches(&html))
            .map(|matches| {
                matches
                .into_iter()
                .map(|m| m.get("value").unwrap().to_string())
                .collect::<Vec<String>>().join("\n")
            })
            .map(|code| {
                    let text= code
                    .replace("\n=\n", "=");
                    // normally, there's only on item but someone could have decided to inform both `license` and `license-file` attributes.
                    // we will take the first one.
                    text.lines().find(|l| l.starts_with("license")).map(|l| {
                        let items = l.split('=').map(|x| x.trim()).collect::<Vec<_>>();
                        (items[0].to_string(), items[1].replace('\"', ""))
                    })
            });

            let retrieved_dependency = match license_info {
                Ok(license_info) => {
                    if let Some((key, value)) = license_info {
                        match key.as_ref() {
                            "license" => {
                                 // TODO: SUPPORT FOR MULTIPLE LICS HERE
                                crates_io_retrieved_dependency(&dependency, Some(vec![value]), None, None, None)
                            }
                            "license-file" => {
                                get_retrieved_dependency_from_license_file(store, crate_url, value, client, &dependency).await
                            }
                            // this should never happen!
                            _ => {
                                tracing::error!("Unknown license key: {}", key);
                                crates_io_retrieved_dependency(&dependency, None, Some("Unexpected license key while parsing cargo.toml"), None, None)
                            }
                        }
                    } else {
                        let user_error = "No information found in Cargo.toml regarding license or license-file.";
                        tracing::error!(
                            "{} Crate {} : {}",
                            user_error,
                            &dependency.name,
                            &dependency.version,
                        );
                        crates_io_retrieved_dependency(&dependency, None, Some(user_error), None, None)
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Error trying to parse docs.rs for crate {} : {}", &dependency.name, &dependency.version);
                    crates_io_retrieved_dependency(
                        &dependency,
                        None,
                        Some("Error trying to parse docs.rs"), None, None
                    )
                }
            };

            Ok::<_, anyhow::Error>(retrieved_dependency)
        }.unwrap_or_else(move |e| {
                let error = e.to_string();
                crates_io_retrieved_dependency(&dep_clone, None, Some(error.as_str()), None, None)
            })
            .boxed()
    }
}

/// Returns the base url of the crate's source code in docs.rs
fn docs_rs_url(dependency_name: &str, dependency_version: &str) -> String {
    format!("https://docs.rs/crate/{dependency_name}/{dependency_version}/source/")
}

/// Returns a `RetrievedDependency` by looking into the Docs.rs declared license file.
/// This function will use `askalono::Store` to determine the kind of license.
/// Note that in the comments of the `RetrievedDependency` there will be a `Comment` with the % score.
async fn get_retrieved_dependency_from_license_file(
    store: Arc<Option<Store>>,
    crate_url: String,
    license: String,
    client: Client,
    dependency: &Dependency,
) -> RetrievedDependency {
    if let Some(store) = store.as_ref() {
        let license_url = format!("{crate_url}{license}");
        if let Ok((license, score)) = get_license_from_docs_rs(&client, store, &license_url).await {
            crates_io_retrieved_dependency(
                dependency,
                Some(vec![license.clone()]),
                None,
                Some(format!(
                    "Our score for this license is {:.2}%.",
                    score * 100.0
                )),
                Some(vec![(license, score)]),
            )
        } else {
            crates_io_retrieved_dependency(
                dependency,
                None,
                Some(&format!(
                    "Not declared in Cargo.toml. Check the url: {license_url}"
                )),
                None,
                None,
            )
        }
    } else {
        tracing::error!("No askalono store present in Rust docs.rs retriever");
        crates_io_retrieved_dependency(
            dependency,
            None,
            Some("No askalono store present"),
            None,
            None,
        )
    }
}

async fn get_license_from_docs_rs(
    client: &Client,
    store: &Store,
    url: &str,
) -> Result<(String, f32), anyhow::Error> {
    let html = client
        .get(url)
        .header("User-Agent", "licensebat-cli (licensebat.com)")
        .send()
        .await?
        .text()
        .await?;

    let pattern = easy_scraper::Pattern::new(
        r#"<div id="source-code"><pre><code>{{value}}</code></pre></div>"#,
    )
    .map_err(Error)?;

    let matches = pattern.matches(&html);
    if matches.is_empty() {
        tracing::error!(%url, "Couldn't get original license from docs.rs");
        Err(Error(String::from("Not found")).into())
    } else {
        let license_html = matches[0]["value"].clone();
        let license = html2text::from_read(license_html.as_bytes(), 3000);
        let result = store.analyze(&TextData::from(license.as_str()));
        Ok((result.name.to_string(), result.score))
    }
}

#[derive(Error, Debug)]
#[error("DocRs Error: {0}")]
struct Error(String);
