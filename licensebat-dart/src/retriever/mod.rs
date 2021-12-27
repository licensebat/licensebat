mod hosted;

pub use hosted::HostedRetriever;
pub trait Retriever: licensebat_core::Retriever {}
