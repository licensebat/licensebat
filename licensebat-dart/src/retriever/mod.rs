//! Retrievers for Dart dependencies
//!
//! A [`Retriever`] is responsible for getting information about a specific dependency.
//! It can use different sources to get the information.
//!
//! [`Retriever`]: crate::retriever::hosted::Retriever

pub mod hosted;

#[doc(inline)]
pub use hosted::Hosted;
