#![allow(clippy::module_name_repetitions)]

//! Core types and traits for [`licensebat-cli`].
//!
//! Libraries authors that want to provide [`Collector`] implmentation should use this crate.
//!
//! [`DependencyCollector`]: crate::collector::DependencyCollector
//! [`DependencyRetriever`]: crate::retriever::DependencyRetriever
//! [`licensebat-cli`]: https://crates.io/crates/licensebat-cli
//! [`licensebat-core`]: http://crates.io/crates/licensebat-core

pub mod collector;
mod dependency;
pub mod licrc;

pub use collector::{Collector, FileCollector};
pub use dependency::*;
