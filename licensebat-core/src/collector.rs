//! Collector traits.
//!
//!

use crate::dependency::RetrievedDependency;
use futures::{future::BoxFuture, stream::FuturesUnordered};
use std::fmt::Debug;

/// Stream of [`RetrievedDependency`]
pub type RetrievedDependencyStream<'a> = FuturesUnordered<BoxFuture<'a, RetrievedDependency>>;

/// Result returning either a [`RetrievedDependencyStream`] or an [`Error`]
pub type RetrievedDependencyStreamResult<'a> = Result<RetrievedDependencyStream<'a>, Error>;

/// Error raised by a collector while parsing/getting the dependencies.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error produced when deserializing a `yaml file` (pubspec.yaml...).
    #[error("Error deserialiazing yaml: {0}")]
    YamlSerde(#[from] serde_yaml::Error),
    /// Error produced when deserializing a `json file` (package-lock.json...).
    #[error("Error deserialiazing json: {0}")]
    JsonSerde(#[from] serde_json::Error),
    /// Error produced when deserializing a `yarn.lock` file.
    #[error("Error parsing yarn.lock file {0}")]
    YarnLock(#[from] yarn_lock_parser::YarnLockError),
    /// Error produced when deserializing a `Cargo.lock` file.
    #[error("Error parsing Cargo.lock file {0}")]
    CargoLock(#[from] cargo_lock::Error),
}

/// Base trait for collectors.
pub trait Collector: Debug + Send + Sync {
    /// Gets the name of the [`Collector`] (npm, dart, rust, go, python...).
    fn get_name(&self) -> String;
}

/// Trait to be implemented for every [`Collector`] dealing with a dependency file.
pub trait FileCollector: Collector {
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
