use super::Retriever;
use askalono::{Store, TextData};
use futures::{
    future::{BoxFuture},
    FutureExt, TryFutureExt,
};
use licensebat_core::{Comment, Dependency, RetrievedDependency, Retriever as CoreRetriever};
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use selectors::Element;
use std::{sync::Arc, vec};
use tracing::instrument;

pub struct HostedRetriever {
    pub store: Arc<Option<Store>>,
    client: Client,
}

impl Retriever for HostedRetriever {}

impl Default for HostedRetriever {
    fn default() -> Self {
        Self::new(None)
    }
}

impl HostedRetriever {
    /// Creates a new [`Retriever`].
    /// If you want to reuse a [`reqwest::Client`]
    /// consider using the [`with_client`] method.
    #[must_use]
    pub fn new(store: Option<Store>) -> Self {
        Self::with_client(Client::new(), store)
    }

    /// Creates a [`Retriever`] reusing a [`reqwest::Client`]
    #[must_use]
    pub fn with_client(client: Client, store: Option<Store>) -> Self {
        Self {
            client,
            store: Arc::new(store),
        }
    }
}

impl Clone for HostedRetriever {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            store: self.store.clone(),
        }
    }
}

impl std::fmt::Debug for HostedRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostedRetriever")
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

impl CoreRetriever for HostedRetriever {
    type Error = reqwest::Error;
    type Future = BoxFuture<'static, Result<RetrievedDependency, Self::Error>>;

    #[instrument(skip(self), level = "debug")]
    fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Future {
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
                            Some("MIT") => retrieved_dependency(&dependency, declared_licenses, None, Some(url), None),
                            _ => {
                                let result = store.analyze(&TextData::from(official_license.as_str()));
                                tracing::debug!(
                                    "Detailed scrapping: SCORE {:?}, LICENSE: {}",
                                    result.score,
                                    result.name
                                );
                                // TODO:  MAGIC NUMBER HERE! THIS SHOULD BE CONFIGURABLE
                                let (license, comment) = if result.score >= 0.8 {
                                    let comment = if Some(result.name.replace("-", " ")) == declared_license {
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
                                )
                            }
                        }
                    } else {
                        retrieved_dependency(&dependency, declared_licenses, None, Some(url), Some(Comment::removable("Using **Pub Dev Generic License**. We couldn't get the original license.")))
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
) -> RetrievedDependency {
    let has_licenses = licenses.is_some();

    RetrievedDependency {
        name: dependency.name.clone(),
        version: dependency.version.clone(),
        url,
        dependency_type: "Dart".to_owned(),
        validated: false,
        is_valid: has_licenses && error.is_none(),
        is_ignored: false,
        error: if error.is_some() {
            error
        } else if has_licenses {
            None
        } else {
            Some("No License".to_owned())
        },
        licenses: if has_licenses {
            licenses
        } else {
            Some(vec!["NO-LICENSE".to_string()])
        },
        comment: if has_licenses || comment.is_some() {
            comment
        } else {
            Some(Comment::removable("Consider **ignoring** this specific dependency. You can also accept the **NO-LICENSE** key to avoid these issues."))
        },
    }
}

/// Returns the imprecise license that pub.dev provides
fn get_imprecise_license(sibling: &ElementRef) -> Option<String> {
    let lic = sibling.inner_html();
    if lic.contains(" (") {
        let imprecise_license = &lic[..lic.find(" (").unwrap()];
        Some(imprecise_license.to_owned())
    } else {
        None
    }
}
