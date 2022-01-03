//! Retrievers for Rust dependencies
//!
//! There are two types of [`Retriever`]s:
//!
//! - [`CratesIo`]: Retrieves information about a dependency from [`crates.io API`](https://crates.io).
//! - [`DocsRs`]: Retrieves information about a dependency from [`docs.rs website`](https://docs.rs).
//!
//! A [`Retriever`] is responsible for getting information about a specific dependency.
//! It can use different sources to get the information.
//!
//! [`Retriever`]: crate::retriever::docs_rs::Retriever

pub mod crates_io;
pub mod docs_rs;
mod utils;

#[allow(deprecated)]
#[doc(inline)]
pub use crates_io::CratesIo;
#[doc(inline)]
pub use docs_rs::DocsRs;
