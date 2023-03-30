//! Collectors for Dart dependencies
//!
//! A [`Collector`] is responsible for extracting the dependencies of a particular project and then get information about them, usually by using a [`Retriever`].
//!
//! It's important to notice that a [`Collector`] is generic over a [`Retriever`] (or several).
//!
//! This is useful so we can mock the [`Retriever`] in our tests.
//!
//! [`Collector`]: licensebat_core::Collector
//! [`FileCollector`]: licensebat_core::FileCollector
//! [`Retriever`]: crate::retriever::hosted::Retriever
mod dart_dependency;

use self::dart_dependency::{DartDependencies, DartDependency};
use crate::retriever::{self, hosted::Retriever};
use futures::prelude::*;
use licensebat_core::{
    collector::{DependencyCollectionResult, DependencyStream, DependencyStreamResult},
    licrc::LicRc,
    Collector, Comment, Dependency, FileCollector,
};
use std::sync::Arc;
use tracing::instrument;

/// Dart Dependency Collector.
///
/// It will parse the content of the `pubspec.yaml` file and get information about the dependencies scraping the `pub.dev` website.
#[derive(Debug, Clone)]
pub struct Dart<R: Retriever> {
    retriever: R,
}

impl Default for Dart<retriever::Hosted> {
    /// Creates a new [`Dart`] [`FileCollector`] that uses a [`retriever::Hosted`].
    fn default() -> Self {
        let retriever = retriever::Hosted::default();
        Self::new(retriever)
    }
}

impl<R: Retriever> Dart<R> {
    /// Creates a new [`Dart`] [`FileCollector`].
    #[must_use]
    pub const fn new(hosted_retriever: R) -> Self {
        Self {
            retriever: hosted_retriever,
        }
    }
}

impl Dart<retriever::Hosted> {
    /// Creates a new [`Dart`] [`FileCollector`] that uses a [`retriever::Hosted`].
    ///
    /// It's basically sintactic sugar to save you from instantiating the [`retriever::Hosted`].
    #[must_use]
    pub fn with_hosted_retriever(
        client: reqwest::Client,
        store: Arc<Option<askalono::Store>>,
    ) -> Self {
        Self::new(retriever::Hosted::new(client, store))
    }
}

impl<R: Retriever> Collector for Dart<R> {
    fn get_name(&self) -> String {
        crate::DART.to_string()
    }
}

impl<R: Retriever> FileCollector for Dart<R> {
    fn get_dependency_filename(&self) -> String {
        "pubspec.lock".to_string()
    }

    #[instrument(skip(self, dependency_file_content), level = "debug")]
    fn get_dependencies(&self, dependency_file_content: &str) -> DependencyCollectionResult {
        let dependencies = serde_yaml::from_str::<DartDependencies>(dependency_file_content)?
            .into_vec_collection();
        Ok(dependencies)
    }

    #[instrument(skip(self, dependencies), level = "debug")]
    fn retrieve_dependencies(&self, dependencies: Vec<Dependency>) -> DependencyStream {
        let futures = dependencies
            .into_iter()
            .map(|dep| get_dependency(dep, &self.retriever).boxed())
            .collect();

        DependencyStream::new(futures)
    }
}

/// Gets a dependency from Dart Pub.
/// It basically transforms a [`Dependency`] into a [`Dependency`].
/// Depending on the type of package ([source]) we will use a different strategy to get the dependency information.
/// There are 3 different sources: sdk, hosted, git.
/// sdk dependencies will be directly validated and ignored.
/// hosted dependencies will be found by scrapping the dart pub website as it seems to be the only solution.
/// git dependencies will require to access GitHub repos, check the path and ref, and look for a LICENSE file.
async fn get_dependency<R: Retriever>(dependency: DartDependency, retriever: &R) -> Dependency {
    match dependency.source.as_ref() {
        "sdk" => resolve_sdk_dependency(&dependency),
        "hosted" => resolve_hosted_dependency(dependency, retriever).await,
        "git" => resolve_git_dependency(&dependency),
        _ => resolve_unknown_dependency(&dependency),
    }
}

/// Resolves to a dependency with 3-Clause BSD License.
///
/// [This Dart document](https://dart.dev/tools/pub/publishing#preparing-to-publish) states that Dart uses this license.
fn resolve_sdk_dependency(dependency: &DartDependency) -> Dependency {
    retrieved_dependency(
        dependency,
        Some(vec!["BSD-3-Clause".to_owned()]),
        None,
        Some("https://github.com/flutter/flutter".to_string()),
        Some(Comment::removable("SDK dependency. **You should accept this dependency**. Consider adding **BSD-3-Clause** to the **.licrc** configuration file.")),
        None,
    )
}

/// Resolves the license by scrapping the Dart pub website and then the license in github if available.
#[allow(clippy::too_many_lines, clippy::single_match_else)]
async fn resolve_hosted_dependency<R: Retriever>(
    dependency: DartDependency,
    retriever: &R,
) -> Dependency {
    if let Ok(dep) = TryInto::<Dependency>::try_into(dependency.clone()) {
        let dep_name = dep.name.clone();
        retriever.get_dependency(dep).await.unwrap_or_else(|e| {
            let url = format!(
                "https://pub.dev/packages/{}/versions/{}",
                dep_name, &dependency.version,
            );
            retrieved_dependency(
                &dependency,
                None,
                Some(e.to_string()),
                Some(url),
                None,
                None,
            )
        })
    } else {
        retrieved_dependency(
            &dependency,
            None,
            Some("No name found for this dependency".to_owned()),
            None,
            None,
            None,
        )
    }
}

fn resolve_git_dependency(dependency: &DartDependency) -> Dependency {
    // TODO: implement git dependencies
    // vamo a ver...
    // here we have the url and the path, and also the sha...
    // so we must get the tree and look for the license...
    // this is complicated as there might be lots of different hosts and we cannot rely on GitHub
    // as the only collector...
    retrieved_dependency(
        dependency,
        None,
        Some("Git source is not supported".to_string()),
        dependency.description.url.clone(),
        Some(Comment::removable("Git projects are not supported yet. We're working on it but there are too many different git hosting providers and supporting private repos is hard. We're marking this as **invalid by default** so you check for yourself the validity of the license. Consider **adding this dependency to the ignored list** in the **.licrc** configuration file if you trust the source.")),
        None,
    )
}

/// Resolves to invalid dependency as we don't support this type for the moment.
fn resolve_unknown_dependency(dependency: &DartDependency) -> Dependency {
    retrieved_dependency(
        dependency,
        None,
        Some(format!("Not supported source {}", dependency.source)),
        None,
        None,
        None,
    )
}

/// Builds a `Dependency`
fn retrieved_dependency(
    dependency: &DartDependency,
    licenses: Option<Vec<String>>,
    error: Option<String>,
    url: Option<String>,
    comment: Option<Comment>,
    suggested_licenses: Option<Vec<(String, f32)>>,
) -> Dependency {
    Dependency::new(
        dependency
            .description
            .name
            .as_ref()
            .map_or("unknown".to_string(), std::borrow::ToOwned::to_owned),
        dependency.version.clone(),
        crate::DART.to_string(),
        url,
        licenses,
        error,
        comment,
        suggested_licenses,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::dart_dependency::Description;
    use crate::retriever;

    const LICENSE_CACHE: &[u8] =
        std::include_bytes!("../../../licensebat-cli/license-cache.bin.zstd");

    #[tokio::test]
    // #[ignore = "only for dev for the moment"]
    async fn integration_check_dependency_supports_license_map_dart_retriever() {
        let dependency_name = "flutter_local_notifications_platform_interface";
        let dep = DartDependency {
            version: "1.0.1".to_string(),
            source: "hosted".to_string(),          // hosted, sdk, git
            dependency: "direct main".to_string(), // direct main, transitive
            description: Description {
                path: None,
                reference: None,
                url: None,
                name: Some(dependency_name.to_string()),
            },
            is_dev: None,
            is_optional: None,
        };

        let store = Arc::new(askalono::Store::from_cache(LICENSE_CACHE).ok());
        let retriever = retriever::Hosted::new(reqwest::Client::new(), store);
        let res = get_dependency(dep, &retriever).await;
        assert_eq!(res.name, dependency_name);
        assert!(res.licenses.is_some());
    }
}
