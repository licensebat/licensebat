#[cfg(test)]
mod integration_npm_retriever {
    use licensebat_core::RetrievedDependency;
    use licensebat_js::retriever::{self, npm::Retriever};

    #[tokio::test]
    async fn it_works() {
        // https://registry.npmjs.org/exit
        let retriever = retriever::Npm::default();
        let dep: RetrievedDependency = retriever.get_dependency("exit", "0.1.2").await;
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "exit");
    }
}
