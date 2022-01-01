use crate::retriever::{self, docs_rs::Retriever};
use cargo_lock::Package;
use futures::FutureExt;
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, Comment, FileCollector,
    RetrievedDependency,
};
use std::{str::FromStr, sync::Arc};
use tracing::instrument;

/// Rust dependency collector
#[derive(Debug)]
pub struct Rust<R: Retriever> {
    docs_rs_retriever: Arc<R>,
}

impl<R: Retriever> Rust<R> {
    #[must_use]
    pub fn new(docs_rs_retriever: R) -> Self {
        Self {
            docs_rs_retriever: Arc::new(docs_rs_retriever),
        }
    }
}

impl Rust<retriever::DocsRs> {
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

    #[instrument(skip(self))]
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult {
        let lockfile = cargo_lock::Lockfile::from_str(dependency_file_content)?;
        let retriever = &self.docs_rs_retriever;

        Ok(lockfile
            .packages
            .into_iter()
            .map(|p| get_dependency(p, retriever.clone()).boxed())
            .collect())
    }
}

async fn get_dependency<R: Retriever>(package: Package, retriever: Arc<R>) -> RetrievedDependency {
    if let Some(source) = package.source {
        // Registries
        if source.is_registry() {
            #[allow(clippy::if_same_then_else)]
            if source.is_default_registry() {
                // this is the only one supported for now
                // TODO: use crates.io retriever
                return retriever
                    .get_dependency(package.name.as_str(), &package.version.to_string())
                    .await;
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
            licenses:  Some(vec!["NO-LICENSE".to_string()]),
            comment: Some(Comment::removable("Git, Local and Remote registries are not supported yet. We're working on it. We're marking this as invalid by default so you can check the validity of the license. Consider adding this dependency to the ignored list in the .licrc configuration file if you trust the source.")),
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{
        future::{ready, BoxFuture},
        StreamExt,
    };

    #[derive(Debug)]
    struct MockRetriever;

    impl Retriever for MockRetriever {
        type Response = BoxFuture<'static, RetrievedDependency>;

        fn get_dependency(&self, dep_name: &str, dep_version: &str) -> Self::Response {
            ready(RetrievedDependency {
                name: dep_name.to_string(),
                version: dep_version.to_string(),
                ..RetrievedDependency::default()
            })
            .boxed()
        }
    }

    fn build_collector() -> Rust<MockRetriever> {
        Rust::new(MockRetriever)
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

        let mut deps = rust.get_dependencies(&lock_content).unwrap();
        assert_eq!(deps.len(), 1);

        let dep = deps.next().await.unwrap();

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

        let mut deps = rust.get_dependencies(&lock_content).unwrap();
        assert_eq!(deps.len(), 1);

        let dep = deps.next().await.unwrap();

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

        let mut deps = rust.get_dependencies(&lock_content).unwrap();
        assert_eq!(deps.len(), 1);

        let dep = deps.next().await.unwrap();

        assert_eq!(rust.get_name(), "rust");
        assert_eq!(rust.get_dependency_filename(), "Cargo.lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "3.0.0");
    }
}
