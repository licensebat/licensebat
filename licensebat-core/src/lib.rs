//! Core types and traits for [`licensebat-cli`].
//!
//! Libraries authors that want to provide [`DependencyCollector`] or [`DependencyRetriever`] implementations
//! should depend on the [`licensebat-core`] crate,.
//!
//! [`DependencyCollector`]: crate::dependency_collector::DependencyCollector
//! [`DependencyRetriever`]: crate::dependency_retriever::DependencyRetriever
//! [`licensebat-cli`]: https://crates.io/crates/licensebat-cli
//! [`licensebat-core`]: http://crates.io/crates/licensebat-core

mod dependency;
pub mod dependency_collector;
pub mod dependency_retriever;
pub mod licrc;

pub use dependency::*;
pub use dependency_collector::DependencyCollector;
pub use dependency_retriever::DependencyRetriever;
