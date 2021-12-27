mod npm;
mod npm_metadata;

pub use npm::NpmRetriever;
/// Marker trait for npm retriever
pub trait Retriever: licensebat_core::Retriever {}
