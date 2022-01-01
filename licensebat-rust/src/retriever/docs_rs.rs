use super::utils::build_crates_io_retrieved_dependency;
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
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response;
}

pub struct DocsRs {
    client: Client,
    store: Arc<Option<Store>>,
}

impl DocsRs {
    /// Creates a new [`DocsRsRetriever`] using the given [`reqwest::Client`].
    #[must_use]
    pub const fn new(client: Client, store: Arc<Option<Store>>) -> Self {
        Self { client, store }
    }
}

impl Default for DocsRs {
    /// Creates a new [`DocsRsRetriever`].
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
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response {
        let dependency = Dependency {
            name: dep_name.to_string(),
            version: dep_version.to_string(),
        };

        let crate_url = docs_rs_url(&dependency.name, &dependency.version);
        let cargo_toml_url = format!("{}Cargo.toml", crate_url);

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
            // docs.rs exposes the content of the Cargo.toml as <code></code> and then applies some JS to format it.
            // When getting the raw html we'll get code that we can convert into toml
            let matches = easy_scraper::Pattern::new(
                r#"<div id="source-code"><pre><code>{{value}}</code></pre></div>"#,
            )
            .map(|pattern| pattern.matches(&html))
            .map(|m| m.get(0)
                .and_then(|m| m.get("value"))
                .and_then(|code| toml::from_str::<toml::Value>(code).ok())
                .and_then(|t| t.get("package")
                .map(|p|
                    (
                        p.get("license").and_then(toml::Value::as_str).map(ToString::to_string), 
                        p.get("license-file").and_then(toml::Value::as_str).map(ToString::to_string),
                    )
                )
            ));

            let retrieved_dependency = match matches {
                Ok(matches) => {
                    // normally, there's only on item but someone could have decided to inform both `license` and `license-file` attributes.
                    match matches {
                        Some((Some(license), _)) => {
                            // TODO: SUPPORT FOR MULTIPLE LICS HERE
                            build_crates_io_retrieved_dependency(&dependency, Some(vec![license]), None, None)
                        },
                        Some((_, Some(license_file))) => {
                            get_retrieved_dependency_from_license_file(store, crate_url, license_file, client, &dependency).await
                        },
                        // no info found or toml parsing failed
                        _ => {
                            let user_error = "No information found in Cargo.toml regarding license or license-file.";
                            tracing::error!(
                                "{} Crate {} : {}",
                                user_error,
                                &dependency.name,
                                &dependency.version,
                            );
                            build_crates_io_retrieved_dependency(&dependency, None, Some(user_error), None)
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error trying to parse docs.rs for crate {} : {}", &dependency.name, &dependency.version);
                    build_crates_io_retrieved_dependency(
                        &dependency,
                        None,
                        Some("Error trying to parse docs.rs"), None
                    )
                }
            };

            Ok::<_, anyhow::Error>(retrieved_dependency)
        }.unwrap_or_else(move |e| {
                let error = e.to_string();
                build_crates_io_retrieved_dependency(&dep_clone, None, Some(error.as_str()), None)
            })
            .boxed()
    }
}

/// Returns the base url of the crate's source code in docs.rs
fn docs_rs_url(dependency_name: &str, dependency_version: &str) -> String {
    format!(
        "https://docs.rs/crate/{}/{}/source/",
        dependency_name, dependency_version
    )
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
        let license_url = format!("{}{}", crate_url, license);
        if let Ok((license, score)) = get_license_from_docs_rs(&client, store, &license_url).await {
            build_crates_io_retrieved_dependency(
                dependency,
                Some(vec![license]),
                None,
                Some(format!(
                    "Our score for this license is {:.2}%.",
                    score * 100.0
                )),
            )
        } else {
            build_crates_io_retrieved_dependency(
                dependency,
                None,
                Some(&format!(
                    "Not declared in Cargo.toml. Check the url: {}",
                    license_url
                )),
                None,
            )
        }
    } else {
        tracing::error!("No askalono store present in Rust docs.rs retriever");
        build_crates_io_retrieved_dependency(
            dependency,
            None,
            Some("No askalono store present"),
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
        tracing::error!("Couldn't get original license from docs.rs");
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
pub struct Error(String);
