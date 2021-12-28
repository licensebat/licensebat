mod crates_io;

pub use crates_io::CratesIoRetriever;
pub trait Retriever: licensebat_core::Retriever {}
