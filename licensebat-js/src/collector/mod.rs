//! Collectors for js/ts dependencies
//!
//! A [`Collector`] is responsible for extracting the dependencies of a particular project and then get information about them, usually by using a [`Retriever`].
//!
//! As the JavaScript ecosystem is huge, there are many different ways to get information about your project's dependencies.
//!
//! This crate currently exposes two different [`FileCollector`] implementations:
//!
//! - [`Npm`]: Reads your dependencies from a `package-lock.json` file.
//! - [`Yarn`]: Reads your dependencies from a `yarn.lock` file.
//!
//! It's important to notice that a [`Collector`] is generic over a [`Retriever`] (or several).
//!
//! This is useful so we can mock the [`Retriever`] in our tests.
//!
//! [`Collector`]: licensebat_core::Collector
//! [`FileCollector`]: licensebat_core::FileCollector
//! [`Retriever`]: crate::retriever::npm::Retriever
//!
mod common;
mod npm;
mod npm_dependency;
mod yarn;

pub use npm::Npm;
pub use yarn::Yarn;
