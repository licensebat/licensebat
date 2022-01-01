pub mod crates_io;
pub mod docs_rs;
mod utils;

#[allow(deprecated)]
pub use crates_io::CratesIo;
pub use docs_rs::DocsRs;
