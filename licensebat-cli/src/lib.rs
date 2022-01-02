#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

mod check;
mod cli;

pub use check::run;
pub use cli::Cli;
