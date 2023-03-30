#[cfg(test)]
mod integration_npm_retriever {
    use licensebat_core::{Dependency, Dependency};
    use licensebat_js::retriever::{self, npm::Retriever};

    #[tokio::test]
    async fn it_works() {
        // https://registry.npmjs.org/exit
        let retriever = retriever::Npm::default();
        let dependency = Dependency {
            name: "exit".to_string(),
            version: "0.1.2".to_string(),
            ..Dependency::default()
        };
        let dep: Dependency = retriever.get_dependency(dependency).await;
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(dep.name, "exit");
    }
}
