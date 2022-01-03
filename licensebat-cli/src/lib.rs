//! A tool to help you verify that your dependencies comply with your license policies.
//!
//! ## What is Licensebat?
//!
//!`Licensebat` is a **CLI** that you can use for free to verify that the dependencies of your project follow your license policies.
//!
//! Let's say, for instance, that you are building a commercial application. In that case, you may consider avoiding the use of some software with a restrictive license like `GPL`.
//!
//! By using `Licensebat`, you can check you don't have any dependency with such a restrictive license. Normally, it will look in all the the dependency tree of your project, so transient dependencies will also be considered.
//!
//! Feel free to use the `CLI` in your CI/CD pipeline, or in your continuous integration server.
//!
//! <pre class="compile_fail" style="white-space:normal;font:inherit;">
//!     <strong>Warning</strong>: licensebat-cli is still in development so you may use it at your own risk.
//! </pre>
//!
//! ## Licensebat GitHub App
//!
//! Aside from the `CLI`, `Licensebat` can be used directly in your GitHub repositories by installing this [GitHub App](https://github.com/marketplace/licensebat).
//!
//! ## Supported languages
//!
//! [![Crates.io](https://img.shields.io/crates/v/licensebat-js?label=licensebat-js&style=flat-square)](https://crates.io/crates/licensebat-js)
//! [![Crates.io](https://img.shields.io/crates/v/licensebat-dart?label=licensebat-dart&style=flat-square)](https://crates.io/crates/licensebat-dart)
//! [![Crates.io](https://img.shields.io/crates/v/licensebat-rust?label=licensebat-rust&style=flat-square)](https://crates.io/crates/licensebat-rust)
//!
//! ## How to use it
//!
//! ```bash
//! licensebat --dependency-file ./Cargo.lock
//! ```
//!
//! That will set all in motion. Take into account that you'll need to have access to the internet for the cli to work properly.
//!
//! You can have more information about the `CLI` by running `licensebat --help`.
//!
//! ```txt
//! USAGE:
//! licensebat [OPTIONS] --dependency-file <dependency-file>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -d, --dependency-file <dependency-file>    Path to the file containing the dependencies of the project. i.e.
//!                                                package-lock.json for npm projects, yarn.lock for yarn projects, etc
//!     -l, --licrc-file <licrc-file>              Path to the .licrc file [default: .licrc]
//! ````
//!
//! ## Logs
//!
//! `Licensebat` uses [`tracing`](https://docs.rs/tracing). You can get logs while running the `CLI` by setting the `RUST_LOG` environment variable.
//!
//! ```bash
//! RUST_LOG=licensebat=info cargo run --dependency-file ./Cargo.lock
//! ```
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]

mod check;
mod cli;

pub use check::run;
#[doc(hidden)]
pub use cli::Cli;
