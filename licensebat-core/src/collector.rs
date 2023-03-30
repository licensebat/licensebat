//! Collector traits.
use crate::dependency::Dependency;
use futures::stream::Stream;
use futures::StreamExt;
use futures::{future::BoxFuture, stream::Iter};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    vec::IntoIter,
};

/// Stream of [`Dependency`]
pub struct DependencyStream<'a> {
    stream: Iter<IntoIter<BoxFuture<'a, Dependency>>>,
}

impl<'a> DependencyStream<'a> {
    /// Creates a new [`DependencyStream`]
    #[must_use]
    pub fn new(futures: Vec<BoxFuture<'a, Dependency>>) -> Self {
        Self {
            stream: futures::stream::iter(futures),
        }
    }

    /// Returns the inner [`Iter`]
    pub fn into_inner(self) -> Iter<IntoIter<BoxFuture<'a, Dependency>>> {
        self.stream
    }
}

impl<'a> Stream for DependencyStream<'a> {
    type Item = BoxFuture<'a, Dependency>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().stream.poll_next_unpin(cx)
    }
}

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

/// Result of a collection of [`Dependency`]
pub type DependencyCollectionResult = Result<Vec<Dependency>, Error>;

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
    /// Returns a collection of [`Dependency`] from the depdency file content.
    /// You can filter the dependencies before retrieving their licenses.
    fn get_dependencies(&self, dependency_file_content: &str) -> DependencyCollectionResult;
    /// Returns a stream of [`Dependency`] ready to be validated.
    /// It accepts a &str with the content of the dependency file
    /// # Errors
    ///
    /// Will return an [`Error`] if the parsing of the dependency file fails.
    fn retrieve_dependencies(
        &self,
        dependencies: impl Iterator<Item = Dependency>,
    ) -> DependencyStream;
}
