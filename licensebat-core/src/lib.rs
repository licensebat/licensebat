//! Core types and traits for [licensebat-cli].
//!
//! Libraries authors that want to provide [`Collector`] implementations should use this crate.
//!
//! [`Collector`] is the central trait of this crate and its implementations will be responsible for retrieving information about the dependencies in form of a [`RetrievedDependency].
//!
//! Note that the [`Collector`] trait is really simple and this is mainly because this trait is intended to serve as the base for more complex traits.
//!
//! In our case, this crate exposes the [`FileCollector`] trait which will use dependency manifest files (such as `package-lock.json`, `yarn.lock`, `Cargo.lock` or `pubspec.yaml`) to extract information about the different dependencies (see [`Dependency`]) and return a stream of [`RetrievedDependency`] or [`RetrievedDependencyStreamResult`].
//!
//! For the moment, [`FileCollector`] is the only used trait in all language implementations but this can easily change. For instance, for the Rust language, it would be nice to use [`cargo-metadata`] instead of having to parse the `Cargo.lock` file. That would make it cheaper to get information about some dependencies as we wouldn't need to use any APIs to fetch license information.
//!
//! Anyway, having those two different approaches would be still valuable because there are scenarios where we don't have access to all the codebase. While using the CLI, through [licensebat-cli] it makes sense to avoid making http requests at all cost, this is not possible in [Licensebat's GitHub Service](https://github.com/marketplace/licensebat) as we don't have access to all the codebase but only a few files.
//!
//! # Features
//!
//! - **licrc-from-file**: Allows to retrieve license information from a file by enabling a `LicRc::from_relative_path` associated function.
//!
//! [licensebat-cli]: https://docs.rs/licensebat-cli/latest/licensebat_cli/
//! [`RetrievedDependencyStreamResult`]: collector::RetrievedDependencyStreamResult
//! [`cargo-metadata`]: https://docs.rs/cargo_metadata/latest/cargo_metadata
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

pub mod collector;
mod dependency;
pub mod licrc;

#[doc(inline)]
pub use collector::{Collector, FileCollector};
pub use dependency::*;
