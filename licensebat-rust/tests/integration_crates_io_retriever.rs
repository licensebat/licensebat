#![allow(deprecated)]
#[cfg(test)]
mod integration_crates_io_retriever {
    use licensebat_core::RetrievedDependency;
    use licensebat_rust::retriever::{self, crates_io::Retriever};

    const LICENSE_CACHE: &[u8] = std::include_bytes!("../../licensebat-cli/license-cache.bin.zstd");

    fn create_store_retriever() -> retriever::CratesIo {
        let store = askalono::Store::from_cache(LICENSE_CACHE).ok();
        retriever::CratesIo::new(reqwest::Client::new(), std::sync::Arc::new(store))
    }

    #[tokio::test]
    async fn default_works_with_declared_license_dependency() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = retriever::CratesIo::default();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "futurify");
    }

    #[tokio::test]
    async fn default_does_not_resolve_non_standard() {
        // https://crates.io/api/v1/crates/ring/0.16.20
        let retriever = retriever::CratesIo::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.16.20").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(None, dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    #[tokio::test]
    async fn default_does_not_resolve_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = retriever::CratesIo::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.17.0-alpha.9").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(None, dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    #[tokio::test]
    async fn store_works_with_declared_license_dependency() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = create_store_retriever();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "futurify");
    }

    #[tokio::test]
    async fn store_does_resolve_non_standard() {
        // https://crates.io/api/v1/crates/ring/0.16.20
        let retriever = create_store_retriever();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.16.20").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(Some(vec!["OpenSSL".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
        assert_eq!(dep.comment.map(|c| c.text.contains("score")), Some(true));
    }

    #[tokio::test]
    async fn store_does_resolve_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = create_store_retriever();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.17.0-alpha.9").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(Some(vec!["OpenSSL".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
        assert_eq!(dep.comment.map(|c| c.text.contains("score")), Some(true));
    }

    // TODO: TEST ERRORS...
}
