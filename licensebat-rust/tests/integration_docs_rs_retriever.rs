#[cfg(test)]
mod integration_docs_rs_retriever_tests {
    use licensebat_core::RetrievedDependency;
    use licensebat_rust::retriever::{self, docs_rs::Retriever};

    const LICENSE_CACHE: &[u8] = std::include_bytes!("../../licensebat-cli/license-cache.bin.zstd");

    fn create_store_retriever() -> retriever::DocsRs {
        let store = askalono::Store::from_cache(LICENSE_CACHE).ok();
        retriever::DocsRs::new(reqwest::Client::new(), std::sync::Arc::new(store))
    }

    #[tokio::test]
    async fn it_works() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await;
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "futurify");
    }

    #[tokio::test]
    async fn it_works_for_non_standard() {
        // https://crates.io/api/v1/crates/ring/0.16.20
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.16.20").await;
        assert_eq!(Some(vec!["non-standard".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    #[tokio::test]
    async fn it_works_for_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = retriever::DocsRs::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.17.0-alpha.9").await;
        assert_eq!(Some(vec!["non-standard".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    // TODO: TEST ERRORS...
}
