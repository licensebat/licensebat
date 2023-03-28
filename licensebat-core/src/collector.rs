//! Collector traits.
use crate::dependency::RetrievedDependency;
use futures::stream::Stream;
use futures::StreamExt;
use futures::{future::BoxFuture, stream::Iter};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    vec::IntoIter,
};

/// Stream of [`RetrievedDependency`]
pub struct RetrievedDependencyStream<'a> {
    stream: Iter<IntoIter<BoxFuture<'a, RetrievedDependency>>>,
}

impl<'a> RetrievedDependencyStream<'a> {
    /// Creates a new [`RetrievedDependencyStream`]
    #[must_use]
    pub fn new(futures: Vec<BoxFuture<'a, RetrievedDependency>>) -> Self {
        Self {
            stream: futures::stream::iter(futures),
        }
    }

    /// Returns the inner [`Iter`]
    pub fn into_inner(self) -> Iter<IntoIter<BoxFuture<'a, RetrievedDependency>>> {
        self.stream
    }
}

impl<'a> Stream for RetrievedDependencyStream<'a> {
    type Item = BoxFuture<'a, RetrievedDependency>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().stream.poll_next_unpin(cx)
    }
}

/// Stream of [`RetrievedDependency`]
// pub type RetrievedDependencyStream<'a> = Iter<IntoIter<BoxFuture<'a, RetrievedDependency>>>;
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
    JsonSerde(#[from] package_lock_json_parser::PackageLockJsonError),
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

/// Trait to be implemented for every [`Collector`] dealing with a dependency file (package-lock.json, pubspec.yaml...).
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
