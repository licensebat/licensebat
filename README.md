# Licensebat

<div align="center">
<img src="https://licensebat.com/images/not_used/logo_orange.png" width="120">
<p>A tool to help you verify that your dependencies comply with your license policies.</p>
</div>

## What is Licensebat?

`Licensebat` is a **CLI** that you can use for free to verify that the dependencies of your project follow your license policies.

Let's say, for instance, that you are building a commercial application. In that case, you may consider avoiding the use of some software with a restrictive license like `GPL`.

By using `Licensebat`, you can check you don't have any dependency with such a restrictive license. Normally, it will look in all the the dependency tree of your project, so transient dependencies will also be considered.

Feel free to use the `CLI` in your CI/CD pipeline, or in your continuous integration server.

**IMPORTANT**: `licensebat-cli` is still in development so you may use it at your own risk.

## Licensebat GitHub App

`Licensebat` can be used directly in your GitHub repositories by installing the [GitHub App](https://github.com/marketplace/licensebat).

Note that this app is **totally free for open source projects**.

It has paid tiers for commercial projects, although you can still leverage the free tier if you don't have many changes in your repository.

## Licensebat CLI

If you want to learn more about the CLI, take a look at the [project's README.md](./licensebat-cli/README.md). There you will find information about how to use it.

## Supported languages

These are the languages that `Licensebat` is supporting right now:

- [JavaScript](./licensebat-js/README.md)
- [TypeScript](./licensebat-js/README.md)
- [Dart](./licensebat-dart/README.md)
- [Rust](./licensebat-rust/README.md)

## Project structure

This is a **monorepo exposing several crates**.

Two of them correspond to the **core traits and the cli**, and the rest are dedicated to **support specific languages**.

Although in this repository there's only a `bin` (i.e the CLI), the language crates are also being use to support the [Licensebat GitHub App](https://github.com/marketplace/licensebat).

### Main crates

[![Crates.io](https://img.shields.io/crates/v/licensebat-core?label=licensebat-core&style=flat-square)](https://crates.io/crates/licensebat-core)
[![Crates.io](https://img.shields.io/crates/v/licensebat-cli?label=licensebat-cli&style=flat-square)](https://crates.io/crates/licensebat-cli)

### Language crates

[![Crates.io](https://img.shields.io/crates/v/licensebat-js?label=licensebat-js&style=flat-square)](https://crates.io/crates/licensebat-js)
[![Crates.io](https://img.shields.io/crates/v/licensebat-dart?label=licensebat-dart&style=flat-square)](https://crates.io/crates/licensebat-dart)
[![Crates.io](https://img.shields.io/crates/v/licensebat-rust?label=licensebat-rust&style=flat-square)](https://crates.io/crates/licensebat-rust)

## Supporting a new language

If you want to support a new language you must create a new `crate` named `licensebat-<language>`. Eventually, it will be published to `crates.io`.

Normally, you should create a `README.md` file in the `licensebat-<language>`. You can copy the `README.md` file from the `licensebat-core` crate.

Generally speaking, these projects will contain, at least, a `Collector`, which will parse the dependency file (`Cargo.lock`, `package.json`...) and retrieve information about the dependencies, most of the times using a `Retriever`. Note that a `Collector` doesn't necessarily need a `Retriever` or even parse the dependency file. There will be times where you'll probably can use a better strategy (e.g. using cargo-metadata instead of parsing `Cargo.lock` and using `crates.io` API).
