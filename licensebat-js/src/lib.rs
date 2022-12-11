//! A library to get information about your JS/TS dependencies
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![warn(missing_docs)]

pub mod collector;
pub mod retriever;
pub use collector::NPM;
