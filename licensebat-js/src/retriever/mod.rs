//! Retrievers for js/ts dependencies
//!
//! A [`Retriever`] is responsible for getting information about a specific dependency.
//! It can use different sources to get the information.
//!
//! [`Retriever`]: crate::retriever::npm::Retriever

pub mod npm;
mod npm_metadata;

#[doc(inline)]
pub use npm::Npm;
