use crate::dependency::RetrievedDependency;
use futures::{future::BoxFuture, stream::FuturesUnordered};
use std::fmt::Debug;

/// Stream of [`RetrievedDependency`]
pub type RetrievedDependencyStream<'a> = FuturesUnordered<BoxFuture<'a, RetrievedDependency>>;

/// [`DependencyCollector`] result returning either a [`RetrievedDependencyStream`] or a [`DependencyCollectorError`]
pub type RetrievedDependencyStreamResult<'a> = Result<RetrievedDependencyStream<'a>, Error>;

// TODO: REMOVE DEPENDENCIES FROM serde_json, serde_value, cargo_lock, yarn_lock
/// Error raised by a collector while parsing/getting the dependencies.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error deserialiazing yaml: {0}")]
    YamlSerde(#[from] serde_yaml::Error),
    #[error("Error deserialiazing json: {0}")]
    JsonSerde(#[from] serde_json::Error),
    #[error("Error parsing yarn.lock file {0}")]
    YarnLock(#[from] yarn_lock_parser::YarnLockError),
    #[error("Error parsing Cargo.lock file {0}")]
    CargoLock(#[from] cargo_lock::Error),
}

/// Trait to be implemented for every dependency collector.
/// It holds information about the dependency file,
/// and exposes the method used to retrieve all the dependencies from it.
pub trait DependencyCollector: Debug + Send + Sync {
    /// Gets the name of the [`DependencyCollector`] (npm, dart, rust, go, python...).
    fn get_name(&self) -> String;
    /// Gets the name of the file holding all the dependencies.
    /// i.e. for npm package-lock.json, for rust cargo.lock
    fn get_dependency_filename(&self) -> String;
    /// Returns a stream of [`RetrievedDependency`] ready to be validated.
    /// It accepts a &str with the content of the dependency file.
    /// # Errors
    ///
    /// Will return an [`Error`] if the parsing of the dependency file fails.
    fn get_dependencies(&self, dependency_file_content: &str) -> RetrievedDependencyStreamResult;
}
