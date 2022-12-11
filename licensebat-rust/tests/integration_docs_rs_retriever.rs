#[cfg(test)]
mod integration_docs_rs_retriever {
    use licensebat_core::RetrievedDependency;
    use licensebat_rust::retriever::{self, docs_rs::Retriever};

    const LICENSE_CACHE: &[u8] = std::include_bytes!("../../licensebat-cli/license-cache.bin.zstd");

    fn create_store_retriever() -> retriever::DocsRs {
        let store = askalono::Store::from_cache(LICENSE_CACHE).ok();
        retriever::DocsRs::new(reqwest::Client::new(), std::sync::Arc::new(store))
    }

    #[tokio::test]
    async fn default_works_with_declared_license_dependency() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await;
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(dep.name, "futurify");
        assert!(dep.suggested_licenses.is_none());
    }

    #[tokio::test]
    async fn default_does_not_resolve_non_standard() {
        // https://crates.io/api/v1/crates/ring/0.16.20
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.16.20").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(None, dep.licenses);
        assert_eq!(dep.name, "ring");
        assert!(dep.suggested_licenses.is_none());
    }

    #[tokio::test]
    async fn default_does_not_resolve_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.17.0-alpha.9").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(None, dep.licenses);
        assert_eq!(dep.name, "ring");
        assert!(dep.suggested_licenses.is_none());
    }

    #[tokio::test]
    async fn store_works_with_declared_license_dependency() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = create_store_retriever();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await;
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "futurify");
        assert!(dep.suggested_licenses.is_none());
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
        let suggested_licenses = dep.suggested_licenses.unwrap();
        assert_eq!(1, suggested_licenses.len());
        assert_eq!(suggested_licenses[0].0, "OpenSSL");
        assert!(suggested_licenses[0].1 > 0.7);
    }

    #[tokio::test]
    async fn store_does_resolve_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = create_store_retriever();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.17.0-alpha.9").await;
        assert_eq!(Some(vec!["OpenSSL".to_string()]), dep.licenses);
        assert_eq!(&dep.dependency_type, licensebat_rust::RUST);
        assert_eq!(dep.name, "ring");
        assert_eq!(dep.comment.map(|c| c.text.contains("score")), Some(true));
        let suggested_licenses = dep.suggested_licenses.unwrap();
        assert_eq!(1, suggested_licenses.len());
        assert_eq!(suggested_licenses[0].0, "OpenSSL");
        assert!(suggested_licenses[0].1 > 0.7);
    }

    // TODO: TEST ERRORS...
}
