//! Collectors for Rust dependencies
//!
//! A [`Collector`] is responsible for extracting the dependencies of a particular project and then get information about them, usually by using a [`Retriever`].
//!
//! This crate currently exposes one [`FileCollector`] implementation that uses a [`Retriever`].
//!
//! It's important to notice that a [`Collector`] is generic over a [`Retriever`] (or several).
//!
//! This is useful so we can mock the [`Retriever`] in our tests.
//!
//! [`Collector`]: licensebat_core::Collector
//! [`FileCollector`]: licensebat_core::FileCollector
//! [`Retriever`]: crate::retriever::docs_rs::Retriever
use crate::retriever::{self, docs_rs::Retriever};
use cargo_lock::Package;
use futures::FutureExt;
use licensebat_core::{
    collector::{RetrievedDependencyStream, RetrievedDependencyStreamResult},
    Collector, Comment, Dependency, FileCollector, RetrievedDependency,
};
use std::{str::FromStr, sync::Arc};
use tracing::instrument;

/// Rust dependency collector.
///
/// It will parse the content of the `Cargo.lock` file and get information about the dependencies.
#[derive(Debug)]
pub struct Rust<R: Retriever> {
    retriever: R,
}

impl<R: Retriever> Rust<R> {
    /// Creates a new [`Rust`] [`FileCollector`].
    ///
    /// # Arguments
    ///
    /// * `retriever` - [`Retriever`] for the docs.rs API.
    #[must_use]
    pub const fn new(retriever: R) -> Self {
        Self { retriever }
    }
}

impl Rust<retriever::DocsRs> {
    /// Creates a new [`Rust`] [`FileCollector`] that uses a [`retriever::DocsRs`].
    ///
    /// It's basically sintactic sugar to save you from instantiating the [`retriever::DocsRs`].
    #[must_use]
    pub fn with_docs_rs_retriever(
        client: reqwest::Client,
        store: Arc<Option<askalono::Store>>,
    ) -> Self {
        Self::new(retriever::DocsRs::new(client, store))
    }
}

impl<R: Retriever> Collector for Rust<R> {
    fn get_name(&self) -> String {
        String::from("rust")
    }
}

impl<R: Retriever> FileCollector for Rust<R> {
    fn get_dependency_filename(&self) -> String {
        String::from("Cargo.lock")
    }

    #[instrument(skip(self, filter_fn))]
    fn get_dependencies(
        &self,
        dependency_file_content: &str,
        filter_fn: &dyn Fn(&Dependency) -> bool,
    ) -> RetrievedDependencyStreamResult {
        let lockfile = cargo_lock::Lockfile::from_str(dependency_file_content)?;
        let futures = lockfile
            .packages
            .into_iter()
            .filter(|p| {
                filter_fn(&Dependency {
                    name: p.name.to_string(),
                    version: p.version.to_string(),
                    is_dev: None,
                    is_optional: None,
                })
            })
            .map(|p| get_dependency(p, &self.retriever).boxed())
            .collect();

        Ok(RetrievedDependencyStream::new(futures))
    }
}

async fn get_dependency<R: Retriever>(package: Package, retriever: &R) -> RetrievedDependency {
    if let Some(source) = package.source {
        // Registries
        if source.is_registry() {
            #[allow(clippy::if_same_then_else)]
            if source.is_default_registry() {
                // this is the only one supported for now
                let dependency = licensebat_core::Dependency {
                    name: package.name.to_string(),
                    version: package.version.to_string(),
                    is_dev: None,
                    is_optional: None,
                };
                return retriever.get_dependency(dependency).await;
            } else if source.is_remote_registry() {
                // remote registry
                // TODO: create remote registry retriever
                // todo!("implement remote registry")
            } else {
                // TODO: create local registry retriever
                // todo!("implement local registry")
            }
        }
        // git
        if source.is_git() {
            // let repo = source.url();
            // let repo_ref = source.git_reference();
            // TODO: Implement this, get cargo.toml from git and check for license
            // TODO: CREATE GIT RETRIEVER
            // return RetrievedDependency::default();
        }
    }
    // this should be filesystem, we can check it source.is_path()
    // this won't ever be implemented
    // unimplemented!()

    // for the moment we're returning a default not implemented.
    RetrievedDependency {
            name: package.name.to_string(),
            version: package.version.to_string(),
            url: None,
            dependency_type: crate::RUST.to_owned(),
            validated: false,
            is_valid: false,
            is_ignored: false,
            error: Some("Crate type not Supported".to_owned()),
            licenses:  None,
            comment: Some(Comment::removable("Git, Local and Remote registries are not supported yet. We're working on it. We're marking this as invalid by default so you can check the validity of the license. Consider adding this dependency to the ignored list in the .licrc configuration file if you trust the source.")),
            suggested_licenses: None,
            is_dev: None,
            is_optional: None,
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{
        future::{ready, BoxFuture},
        StreamExt,
    };
    use licensebat_core::Dependency;

    #[derive(Debug)]
    struct MockRetriever;

    impl Retriever for MockRetriever {
        type Response = BoxFuture<'static, RetrievedDependency>;

        fn get_dependency(&self, dependency: Dependency) -> Self::Response {
            ready(RetrievedDependency {
                name: dependency.name.to_string(),
                version: dependency.version.to_string(),
                ..RetrievedDependency::default()
            })
            .boxed()
        }
    }

    fn build_collector() -> Rust<MockRetriever> {
        Rust::new(MockRetriever)
    }

    fn filter_fn(_: &Dependency) -> bool {
        true
    }

    #[tokio::test]
    async fn it_works_for_crates_registry() {
        let rust = build_collector();
        let lock_content = r#"
        [[package]]
        name = "mime"
        version = "0.3.16"
        source = "registry+https://github.com/rust-lang/crates.io-index"
        checksum = "2a60c7ce501c71e03a9c9c0d35b861413ae925bd979cc7a4e30d060069aaac8d"
        "#;

        let mut deps = rust
            .get_dependencies(&lock_content, &filter_fn)
            .unwrap()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(deps.len(), 1);

        let dep = deps[0].as_mut().fuse().await;

        assert_eq!(rust.get_name(), "rust");
        assert_eq!(rust.get_dependency_filename(), "Cargo.lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "0.3.16");
    }

    #[tokio::test]
    async fn it_works_for_crates_registry_with_special_version() {
        let rust = build_collector();
        let lock_content = r#"
        [[package]]
        name = "mime"
        version = "3.0.0-beta.4"
        source = "registry+https://github.com/rust-lang/crates.io-index"
        checksum = "2a60c7ce501c71e03a9c9c0d35b861413ae925bd979cc7a4e30d060069aaac8d"
        "#;

        let mut deps = rust
            .get_dependencies(&lock_content, &filter_fn)
            .unwrap()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(deps.len(), 1);

        let dep = deps[0].as_mut().fuse().await;

        assert_eq!(rust.get_name(), "rust");
        assert_eq!(rust.get_dependency_filename(), "Cargo.lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "3.0.0-beta.4");
    }

    #[tokio::test]
    async fn git_is_not_implemented() {
        let rust = build_collector();
        let lock_content = r#"
        [[package]]
        name = "mime"
        version = "3.0.0"
        source = "git+https://github.com/rust-lang/crates.io-index"
        checksum = "2a60c7ce501c71e03a9c9c0d35b861413ae925bd979cc7a4e30d060069aaac8d"
        "#;

        let mut deps = rust
            .get_dependencies(&lock_content, &filter_fn)
            .unwrap()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(deps.len(), 1);

        let dep = deps[0].as_mut().fuse().await;

        assert_eq!(rust.get_name(), "rust");
        assert_eq!(rust.get_dependency_filename(), "Cargo.lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "3.0.0");
    }
}
