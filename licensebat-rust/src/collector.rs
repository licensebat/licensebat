use cargo_lock::Package;
use futures::FutureExt;
use licensebat_core::{
    collector::RetrievedDependencyStreamResult, Collector, FileCollector, RetrievedDependency,
    Retriever,
};

use std::{str::FromStr, sync::Arc};
use tracing::instrument;

/// Rust dependency collector
#[derive(Debug)]
pub struct RustCollector<R: Retriever> {
    crates_io_retriever: Arc<R>,
}

impl<R: Retriever> RustCollector<R> {
    pub fn new(crates_io_retriever: R) -> Self {
        Self {
            crates_io_retriever: Arc::new(crates_io_retriever),
        }
    }
}

impl<R: Retriever> Collector for RustCollector<R> {
    fn get_name(&self) -> String {
        String::from("rust")
    }
}

impl<R: Retriever> FileCollector for RustCollector<R> {
    fn get_dependency_filename(&self) -> String {
        String::from("Cargo-lock")
    }

    #[instrument(skip(self))]
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult {
        let lockfile = cargo_lock::Lockfile::from_str(dependency_file_content)?;
        let retriever = &self.crates_io_retriever;

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
            if source.is_default_registry() {
                // this is the only one supported for now
                // TODO: use crates.io retriever
                return retriever
                    .get_dependency(package.name.as_str(), &package.version.to_string())
                    .map(std::result::Result::unwrap) // TODO: or else
                    .await;
            } else if source.is_remote_registry() {
                // remote registry
                // TODO: create remote registry retriever
                todo!("implement remote registry")
            } else {
                // TODO: create local registry retriever
                todo!("implement local registry")
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
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use licensebat_core::retriever::EmptyRetriever;

    #[tokio::test]
    async fn it_works_for_crates_registry() {
        let rust = RustCollector::new(EmptyRetriever::default());
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
        assert_eq!(rust.get_dependency_filename(), "Cargo-lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "0.3.16");
    }

    #[tokio::test]
    async fn it_works_for_crates_registry_with_special_version() {
        let rust = RustCollector::new(EmptyRetriever::default());
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
        assert_eq!(rust.get_dependency_filename(), "Cargo-lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "3.0.0-beta.4");
    }

    #[tokio::test]
    #[should_panic]
    async fn git_is_not_implemented() {
        let rust = RustCollector::new(EmptyRetriever::default());
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
        assert_eq!(rust.get_dependency_filename(), "Cargo-lock");
        assert_eq!(dep.name, "mime");
        assert_eq!(dep.version, "3.0.0");
    }
}
