#[cfg(test)]
mod tests {
    use askalono::Store;
    use licensebat_core::Retriever;
    use licensebat_retriever_dart::HostedRetriever;

    const LICENSE_CACHE: &[u8] = std::include_bytes!("../../datasets/license-cache.bin.zstd");

    fn create_retriever() -> HostedRetriever {
        let store = Store::from_cache(LICENSE_CACHE).ok();
        HostedRetriever::new(store)
    }

    #[tokio::test]
    async fn it_works_with_discontinued_packages() {
        // https://pub.dev/packages/flare_dart/versions/2.3.3
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("flare_dart", "2.3.3")
            .await
            .unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert!(dep.comment.is_some());
        assert_eq!(dep.name, "flare_dart");
    }

    #[tokio::test]
    async fn it_works_with_different_declared_license() {
        // https://pub.dev/packages/flutter_local_notifications_platform_interface/versions/1.0.1
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("flutter_local_notifications_platform_interface", "1.0.1")
            .await
            .unwrap();
        assert_eq!(Some(vec!["BSD-3-Clause".to_string()]), dep.licenses);
        assert!(dep.comment.is_some());
        assert_eq!(dep.name, "flutter_local_notifications_platform_interface");
    }

    #[tokio::test]
    async fn it_works_with_different_declared_license_and_nullsafety_label() {
        // https://pub.dev/packages/file/versions/6.0.0-nullsafety.2
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("file", "6.0.0-nullsafety.2")
            .await
            .unwrap();
        assert_eq!(Some(vec!["BSD-3-Clause".to_string()]), dep.licenses);
        assert!(dep.comment.is_some());
        assert_eq!(dep.name, "file");
    }

    // this package does not qualify for the test
    #[ignore]
    #[tokio::test]
    async fn it_uses_analysis_when_no_declared_license_and_result_is_over_79() {
        // https://pub.dev/packages/fake_async/versions/1.2.0-nullsafety.1
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("fake_async", "1.1.0-nullsafety.1")
            .await
            .unwrap();
        assert_eq!(Some(vec!["Apache-2.0".to_string()]), dep.licenses);
        assert_eq!(
            dep.comment.map(|x| x.text.contains("Our score for")),
            Some(true)
        );
        assert_eq!(dep.name, "fake_async");
    }

    #[tokio::test]
    async fn it_uses_analysis_when_no_declared_license_and_result_is_over_79_and_pubdev_layout_is_different(
    ) {
        // https://pub.dev/packages/random_color/versions/1.0.3
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("random_color", "1.0.3")
            .await
            .unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert_eq!(
            dep.comment.map(|x| x.text.contains("Our score for")),
            Some(true)
        );
        assert_eq!(dep.name, "random_color");
    }

    // this library analysis is 93...
    #[ignore]
    #[tokio::test]
    async fn it_uses_analysis_when_no_declared_license_and_result_is_below_80() {
        // https://pub.dev/packages/vector_math/versions/2.0.8
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("vector_math", "2.0.8")
            .await
            .unwrap();
        assert_eq!(Some(vec!["NO-LICENSE".to_string()]), dep.licenses);
        assert!(dep.comment.is_some());
        assert_eq!(
            dep.comment
                .map(|x| x.text.contains("Pub Dev Generic License")),
            Some(true)
        );
        assert_eq!(dep.name, "vector_math");
    }

    #[tokio::test]
    async fn it_works_with_mit() {
        // https://pub.dev/packages/flutter_isolate/versions/1.0.0+14
        let retriever = create_retriever();
        let dep = retriever
            .get_dependency("flutter_isolate", "1.0.0+14")
            .await
            .unwrap();
        assert_eq!(Some(vec!["MIT".to_string()]), dep.licenses);
        assert!(dep.comment.is_none());
        assert_eq!(dep.name, "flutter_isolate");
    }
}
