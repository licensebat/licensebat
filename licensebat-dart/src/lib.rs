//! A library to get information about your Dart dependencies
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![warn(missing_docs)]

pub mod collector;
pub mod retriever;
/// String used to identify the type of dependency
pub const DART: &str = "dart";
