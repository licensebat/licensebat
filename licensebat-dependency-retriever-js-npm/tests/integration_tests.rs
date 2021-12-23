#[cfg(test)]
mod tests {
    use licensebat_core::{DependencyRetriever, RetrievedDependency};
    use licensebat_dependency_retriever_js_npm::NpmDependencyRetriever;

    #[tokio::test]
    async fn it_works() {
        // https://registry.npmjs.org/exit
        let retriever = NpmDependencyRetriever::default();
        let dep: RetrievedDependency = retriever.get_dependency("exit", "0.1.2").await.unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "exit");
    }
}
