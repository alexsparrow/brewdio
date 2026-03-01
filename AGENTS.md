# Agents

## Building the WASM library

When making changes to any Rust crate (`brewdio-core`, `brewdio-persistence`, `brewdio-wasm`), rebuild the WASM package before testing the web UI:

```sh
bun run build:wasm
```

This runs an unoptimised debug build (`wasm-pack build --dev`) which is significantly faster than the release build used in CI. The release build applies LTO, wasm-opt, and size optimisations that are unnecessary for local development.

The output goes to `brewdio-wasm/pkg/` which is linked as a workspace dependency (`brewdio-wasm`) in the web UI.
