#[cfg(test)]
mod tests {
    use licensebat_core::{RetrievedDependency, Retriever};
    use licensebat_retriever_js::NpmRetriever;

    #[tokio::test]
    async fn it_works() {
        // https://registry.npmjs.org/exit
        let retriever = NpmRetriever::default();
        let dep: RetrievedDependency = retriever.get_dependency("exit", "0.1.2").await.unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "exit");
    }
}
