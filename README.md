# Licensebat

<div align="center">
<img src="https://licensebat.com/images/not_used/logo_orange.png" width="120">
<p>A tool to help you verify that your dependencies comply with your license policies.</p>
</div>

## Crates

This is a monorepo exposing several crates.

Two of them correspond to the core traits and the cli, and the rest are dedicated to support specific languages.

Although in this repository there's only a `bin` (i.e the cli), the language crates are also being use to support the [Licensebat GitHub App](https://github.com/marketplace/licensebat).

### Main crates

![Crates.io](https://img.shields.io/crates/v/licensebat-core?label=licensebat-core&style=flat-square)
![Crates.io](https://img.shields.io/crates/v/licensebat-cli?label=licensebat-cli&style=flat-square)

### Language crates

![Crates.io](https://img.shields.io/crates/v/licensebat-js?label=licensebat-js&style=flat-square)
![Crates.io](https://img.shields.io/crates/v/licensebat-dart?label=licensebat-dart&style=flat-square)
![Crates.io](https://img.shields.io/crates/v/licensebat-rust?label=licensebat-rust&style=flat-square)

## Supporting a new language

If you want to support a new language you must create a new `crate` named `licensebat-<language>`. Eventually, it will be published to `crates.io`.

Normally, you should create a `README.md` file in the `licensebat-<language>`. You can copy the `README.md` file from the `licensebat-core` crate.

Generally speaking, these projects will contain, at least, a `Collector`, which will parse the dependency file (`Cargo.lock`, `package.json`...) and retrieve information about the dependencies, most of the times using a `Retriever`. Note that a `Collector` doesn't necessarily need a `Retriever` or even parse the dependency file. There will be times where you'll probably can use a better strategy (e.g. using cargo-metadata instead of parsing `Cargo.lock` and using `crates.io` API).
