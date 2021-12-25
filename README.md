# Licensebat

All docs here are temporary.

## Thougths

For the moment, it seems it makes sense to have all the `collectors` sharing the same trait. That doesn't seem to scale for `retrievers`

## Conventions

If you want to support a new language you must create a new `crate` named `licensebat-<language>`. It will be published to `crates.io`.

Normally, you should create a `README.md` file in the `licensebat-<language>` crate.

Generally speaking, these projects will contain:

- A `Collector`, which will parse the dependency file (`Cargo.toml` or `package.json`) and retrieve information about the dependencies, most of the times using a `Retriever`. Note that a `Collector` doesn't necessarily need a `Retriever` or even parse the dependency file. There will be times where you'll probably can use a better strategy (e.g. using cargo metadata instead of parsing `Cargo.lock` and using `crates.io` API).

### Collectors

- Each project can have more than one `Collector`.
- Use `Collector` as a suffix. Not for the .rs file, just the `struct`.

### Retrievers

- Each project can have more than one `Retriever`.
- Use `Retriever` as a suffix for different retriever types (i.e. NpmRetriever, GitRetriever, etc.). Not for the .rs file, just the `struct`.
