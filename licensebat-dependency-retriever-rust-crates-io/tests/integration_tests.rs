#[cfg(test)]
mod tests {
    use licensebat_core::{DependencyRetriever, RetrievedDependency};
    use licensebat_dependency_retriever_rust_crates_io::RustCratesIoDependencyRetriever;

    #[tokio::test]
    async fn it_works() {
        // https://crates.io/api/v1/crates/futurify/0.2.0
        let retriever = RustCratesIoDependencyRetriever::default();
        let dep: RetrievedDependency = retriever.get_dependency("futurify", "0.2.0").await.unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "futurify");
    }

    #[tokio::test]
    async fn it_works_for_non_standard() {
        // https://crates.io/api/v1/crates/ring/0.16.20
        let retriever = RustCratesIoDependencyRetriever::default();
        let dep: RetrievedDependency = retriever.get_dependency("ring", "0.16.20").await.unwrap();
        assert_eq!(Some(vec!["non-standard".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    #[tokio::test]
    async fn it_works_for_non_standard_yanked() {
        // https://crates.io/api/v1/crates/ring/0.17.0-alpha.9
        let retriever = RustCratesIoDependencyRetriever::default();
        let dep: RetrievedDependency = retriever
            .get_dependency("ring", "0.17.0-alpha.9")
            .await
            .unwrap();
        assert_eq!(Some(vec!["non-standard".to_string()]), dep.licenses);
        assert_eq!(dep.name, "ring");
    }

    // TODO: TEST ERRORS...
}
