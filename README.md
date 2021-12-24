# Licensebat

All docs here are temporary.

## Thougths

For the moment, it seems it makes sense to have all the `collectors` sharing the same trait. That doesn't seem to scale for `retrievers`

## Conventions

### Collectors

- The project must be named `licensebat-collector-<lang>`.
- Each project can have more than one `Collector`.
- Don't use `Collector` as a suffix.
- Keep the name short and simple.
- Prefer fully qualified name for `Collectors`.

### Retrievers

- The project must be named `licensebat-retriever-<lang>`.
- Each project can have more than one `Retriever`.
- Use `Retriever` as a suffix for different retriever types (i.e. NpmRetriever, GitRetriever, etc.).
