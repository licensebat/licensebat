//! [`Retriever`] that uses the [pub.dev website].
//!
//! Here you can find both the trait and the implementation.
//!
//! Usually, [`Collectors`](licensebat_core::Collector) are generic over a [`Retriever`] (or several). This comes in handy for mocking the [`Retriever`] in our tests.
//!
//! [`Retriever`]: crate::retriever::hosted::Retriever
//! [pub.dev website]: https://pub.dev/

use askalono::{Store, TextData};
use futures::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use licensebat_core::{Comment, Dependency, RetrievedDependency};
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use selectors::Element;
use std::{sync::Arc, vec};
use tracing::instrument;

/// Trait used by the [`Hosted`] struct to retrieve dependencies.
pub trait Retriever: Send + Sync + std::fmt::Debug {
    /// The associated error which can be returned.
    type Error: std::fmt::Debug + std::fmt::Display;
    /// Future that resolves to a [`RetrievedDependency`].
    type Response: Future<Output = Result<RetrievedDependency, Self::Error>> + Send;
    /// Validates dependency's information from the original source.
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response;
}

/// [`pub.dev`] [`Retriever`] implementation.
///
/// It uses [`reqwest::Client`] to scrap the [`pub.dev`] website and retrieve the metadata of a dependency.
///
/// You can provide yourself an instance of [`reqwest::Client`] by using the [`Hosted::new`] constructor.
///
/// If you use [`Hosted::default`], it will instantiate a new [`reqwest::Client`] under the hood.
///
/// [`pub.dev`]: https://pub.dev
pub struct Hosted {
    client: Client,
    store: Arc<Option<Store>>,
}

impl Hosted {
    /// Creates a new [`Retriever`].
    /// If you want to reuse a [`reqwest::Client`] pool consider using the [`Hosted::new`] method.
    #[must_use]
    pub fn new(client: Client, store: Arc<Option<Store>>) -> Self {
        Self { client, store }
    }
}

impl Default for Hosted {
    /// Creates a new [`Retriever`] using the given [`reqwest::Client`].
    /// If you don't want to pass a [`reqwest::Client`] instance, consider using the [`Hosted::default`] method.
    fn default() -> Self {
        Self::new(Client::new(), Arc::new(None))
    }
}

impl Clone for Hosted {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            store: self.store.clone(),
        }
    }
}

impl std::fmt::Debug for Hosted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hosted")
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

impl Retriever for Hosted {
    type Error = reqwest::Error;
    type Response = BoxFuture<'static, Result<RetrievedDependency, Self::Error>>;

    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response {
        let url = format!(
            "https://pub.dev/packages/{}/versions/{}",
            dep_name, dep_version,
        );

        let dependency = Dependency {
            name: dep_name.to_string(),
            version: dep_version.to_string(),
        };

        let store = self.store.clone();

        self.client
            .get(format!("{}/license", url))
            .send()
            .and_then(reqwest::Response::text)
            .map(move |html| {
                html.map(|html| {
                    let url = url.clone();
                    // scrape the html looking for the license
                    let document = Html::parse_document(&html);
                    let declared_license = Selector::parse(r#"h3[class="title"]"#).ok()
                        .and_then(|selector| {
                            document
                                .select(&selector)
                                .filter(|s| s.inner_html() == "License")
                                .map(|s| s.next_sibling_element().and_then(|sibling| get_imprecise_license(&sibling)))
                                .next()
                                .flatten()
                        });

                    let mut official_license = Selector::parse(r#".detail-container.detail-body-main .highlight pre"#).ok().and_then( |selector| {
                        document.select(&selector).map(|s| s.inner_html()).next()
                    });

                    // there are some licenses that are not printed in the same way in pub.dev
                    if official_license.is_none() {
                        official_license = Selector::parse(r#".detail-container.detail-body-main .tab-content"#).ok().and_then( |selector| {
                            document.select(&selector).map(|s| s.inner_html()).next()
                        });
                    }

                    let declared_licenses = declared_license.clone().map(|x| vec![x]);

                    if let (Some(official_license), Some(store)) = (official_license, store.as_ref()) {
                        // Some licenses, like BSD are represented in an imprecise way in pub dev,
                        // so we must scrape the github license file.
                        // Nevertheless, there are some of them that ar ok, as MIT,
                        // so we'll short circuit these ones.
                        #[allow(clippy::single_match_else)]
                        match declared_license.as_deref() {
                            Some("MIT") => retrieved_dependency(&dependency, declared_licenses, None, Some(url), None, None),
                            _ => {
                                let result = store.analyze(&TextData::from(official_license.as_str()));
                                tracing::debug!(
                                    "Detailed scrapping: SCORE {:?}, LICENSE: {}",
                                    result.score,
                                    result.name
                                );
                                // TODO:  MAGIC NUMBER HERE! THIS SHOULD BE CONFIGURABLE
                                let (license, comment) = if result.score >= 0.8 {
                                    let comment = if Some(result.name.replace('-', " ")) == declared_license {
                                        None
                                    } else {
                                        let comment = format!(
                                            "Pub Dev license: {}. Our score for **{}** is **{:.2}%**.",
                                            declared_license.unwrap_or_else(|| "NOT DECLARED".to_owned()),
                                            result.name,
                                            result.score * 100.0
                                        );
                                        Some(comment)
                                    };
                                    (Some(result.name.to_string()), comment)
                                } else {
                                    let comment = format!(
                                        "Using **Pub Dev Generic License**. Our analysis, though, estimated that it could be **{}** with a **{:.2}%** score.",
                                        result.name,
                                        result.score * 100.0
                                    );
                                    (declared_license.clone(), Some(comment))
                                };

                                retrieved_dependency(
                                    &dependency,
                                    license.map(|l| vec![l]),
                                    None,
                                    Some(url),
                                    comment.map(Comment::non_removable),
                                    Some(vec![(result.name.to_string(), result.score)])
                                )
                            }
                        }
                    } else {
                        retrieved_dependency(&dependency, declared_licenses, None, Some(url), Some(Comment::removable("Using **Pub Dev Generic License**. We couldn't get the original license.")), None)
                    }
                })
            }).boxed()
    }
}

fn retrieved_dependency(
    dependency: &Dependency,
    licenses: Option<Vec<String>>,
    error: Option<String>,
    url: Option<String>,
    comment: Option<Comment>,
    suggested_licenses: Option<Vec<(String, f32)>>,
) -> RetrievedDependency {
    RetrievedDependency::new(
        dependency.name.clone(),
        dependency.version.clone(),
        crate::DART.to_owned(),
        url,
        licenses,
        error,
        comment,
        suggested_licenses,
    )
}

/// Returns the imprecise license that pub.dev provides
fn get_imprecise_license(sibling: &ElementRef) -> Option<String> {
    let lic = sibling.inner_html();
    if lic.contains(" (") {
        let imprecise_license = &lic[..lic.find(" (").unwrap()];
        if imprecise_license.starts_with("<img") && imprecise_license.contains("\">") {
            return Some(
                imprecise_license[imprecise_license.find("\">").unwrap() + 2..].to_owned(),
            );
        }
        Some(imprecise_license.to_owned())
    } else {
        None
    }
}
