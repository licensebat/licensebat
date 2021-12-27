//! Core types and traits for [`licensebat-cli`].
//!
//! Libraries authors that want to provide [`DependencyCollector`] or [`DependencyRetriever`] implementations
//! should depend on the [`licensebat-core`] crate,.
//!
//! [`DependencyCollector`]: crate::collector::DependencyCollector
//! [`DependencyRetriever`]: crate::retriever::DependencyRetriever
//! [`licensebat-cli`]: https://crates.io/crates/licensebat-cli
//! [`licensebat-core`]: http://crates.io/crates/licensebat-core

pub mod collector;
mod dependency;
pub mod licrc;
pub mod retriever;

pub use collector::{Collector, FileCollector};
pub use dependency::*;
pub use retriever::Retriever;
