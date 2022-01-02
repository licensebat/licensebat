#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

mod check;
mod cli;

pub use check::run;
pub use cli::Cli;
