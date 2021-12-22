use crate::Cli;
use futures::StreamExt;
use licensebat_core::{licrc::LicRc, DependencyCollector, RetrievedDependency};

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("Error reading dependency file: {0}")]
    DependencyFile(#[from] std::io::Error),
}

pub async fn run(cli: Cli) -> anyhow::Result<Vec<RetrievedDependency>> {
    tracing::info!(
        dependency_file = %cli.dependency_file,
        "Licensebat running! Using {}", cli.dependency_file
    );

    // 1 .get information from .licrc file
    tracing::debug!("Reading .licrc file");
    let licrc = LicRc::from_relative_path(cli.licrc_file)?;

    // 2. get content of the dependency file
    tracing::debug!("Getting dependency file content");
    let dep_file_content = get_dep_file_content(&cli.dependency_file).await?;

    // 3. use a collector to get a stream of  dependencies
    tracing::debug!("Getting dependencies from file content");
    let npm = licensebat_dependency_collector_js::Npm::default();
    let mut stream = npm.get_dependencies(&dep_file_content)?;

    // 4. validate the dependencies according to the .licrc config
    tracing::debug!("Validating dependencies");
    let mut validated_deps = vec![];
    while let Some(mut dependency) = stream.next().await {
        // do the validation here
        licrc.validate(&mut dependency);
        validated_deps.push(dependency);
    }

    tracing::info!("Done!");
    Ok(validated_deps)
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
