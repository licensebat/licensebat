use crate::Cli;
use futures::StreamExt;
use licensebat_core::{licrc::LicRc, FileCollector, RetrievedDependency};
use std::sync::Arc;

const LICENSE_CACHE: &[u8] = std::include_bytes!("../license-cache.bin.zstd");

#[derive(Debug, thiserror::Error)]
enum CheckError {
    #[error("Error reading dependency file: {0}")]
    DependencyFile(#[from] std::io::Error),
}

/// Result of the dependency validation.
pub struct RunResult {
    /// The [`LicRc`] file.
    pub licrc: LicRc,
    /// The validated dependencies.
    pub dependencies: Vec<RetrievedDependency>,
}

/// Checks the dependencies of a project.
///
/// This is the main entry point of the CLI.
///
/// # Errors
///
/// Errors can be caused by many causes, including:
/// - Reading a dependency manifest file (package-lock.json, yarn.lock, etc.)
/// - Reading the .licrc file
pub async fn run(cli: Cli) -> anyhow::Result<RunResult> {
    tracing::info!(
        dependency_file = %cli.dependency_file,
        "Licensebat running! Using {}", cli.dependency_file
    );

    // 0. spdx store & http client
    let store = Arc::new(askalono::Store::from_cache(LICENSE_CACHE).ok());
    let client = reqwest::Client::builder()
        .no_proxy()
        // .danger_accept_invalid_certs(true)
        // .pool_idle_timeout(None)
        .build()
        .expect("Failed to build HTTP client");

    // 1 .get information from .licrc file
    tracing::debug!("Reading .licrc file");
    let licrc = LicRc::from_relative_path(cli.licrc_file)?;

    // 2. get content of the dependency file
    tracing::debug!("Getting dependency file content");
    let dep_file_content = get_dep_file_content(&cli.dependency_file).await?;

    // 3. create collectors
    tracing::debug!("Building collectors");
    let npm_retriever = licensebat_js::retriever::Npm::new(client.clone());
    let npm_collector = licensebat_js::collector::Npm::new(npm_retriever.clone());
    let yarn_collector = licensebat_js::collector::Yarn::new(npm_retriever);
    let rust_collector =
        licensebat_rust::collector::Rust::with_docs_rs_retriever(client.clone(), store.clone());
    let dart_collector = licensebat_dart::collector::Dart::with_hosted_retriever(client, store);

    let file_collectors: Vec<Box<dyn FileCollector>> = vec![
        Box::new(npm_collector),
        Box::new(yarn_collector),
        Box::new(rust_collector),
        Box::new(dart_collector),
    ];

    // 4. get dependency stream
    // TODO: this filter function is already calculating ingored dependencies.
    // we're also doing it in the licrc validator. Ideally we should only do it once.
    let filter = |dependency: &licensebat_core::Dependency| {
        let is_dev = dependency.is_dev.unwrap_or_default();
        let is_optional = dependency.is_optional.unwrap_or_default();

        if licrc.behavior.do_not_show_dev_dependencies && is_dev {
            return false;
        }
        if licrc.behavior.do_not_show_optional_dependencies && is_optional {
            return false;
        }

        let is_ignored = licrc
            .dependencies
            .ignored
            .as_ref()
            .unwrap_or(&vec![])
            .contains(&dependency.name);

        if licrc.behavior.do_not_show_ignored_dependencies && is_ignored {
            if is_ignored {
                return false;
            }
            if licrc.dependencies.ignore_dev_dependencies && is_dev {
                return false;
            }
            if licrc.dependencies.ignore_optional_dependencies && is_optional {
                return false;
            }
        }

        true
    };

    let mut stream = file_collectors
        .iter()
        .find(|c| cli.dependency_file.contains(&c.get_dependency_filename()))
        .and_then(|c| c.get_dependencies(&dep_file_content, &filter).ok())
        .expect(
            format!(
                "No collector found for dependency file {}",
                cli.dependency_file
            )
            .as_str(),
        )
        .buffer_unordered(licrc.behavior.retriever_buffer_size.unwrap_or(100));

    // 5. validate the dependencies according to the .licrc config
    tracing::debug!("Validating dependencies");
    let mut validated_deps = vec![];

    while let Some(mut dependency) = stream.next().await {
        // do the validation here
        licrc.validate(&mut dependency);
        validated_deps.push(dependency);
    }

    tracing::info!("Done!");
    Ok(RunResult {
        licrc,
        dependencies: validated_deps,
    })
}

async fn get_dep_file_content(dependency_file: &str) -> Result<String, CheckError> {
    async {
        let dep_file_path = std::env::current_dir()?.join(dependency_file);
        let dep_file_content = tokio::fs::read_to_string(dep_file_path).await?;
        Ok(dep_file_content)
    }
    .await
    .map_err(CheckError::DependencyFile)
}
