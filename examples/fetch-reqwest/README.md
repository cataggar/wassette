# Fetch Example (Rust, reqwest)

This example demonstrates how to fetch content from a URL using a Wassette component written in Rust, using `reqwest` for HTTP.

For more information on installing Wassette, please see the [installation instructions](https://github.com/microsoft/wassette?tab=readme-ov-file#installation).

## Usage

To use this component, load it from a local path or OCI registry and provide a URL to fetch.

**Build the component:**

```sh
just build
```

The resulting Wasm will be in `target/wasm32-wasip2/debug/fetch_reqwest.wasm` (or `release/` when built with `mode=release`).

**Grant network policy:**

Use `policy.yaml` as a starting point to allow hosts.

**Fetch content:**

Ask your agent to load the component and fetch:

```text
Please load the component from /absolute/path/to/examples/fetch-reqwest/target/wasm32-wasip2/debug/fetch_reqwest.wasm
Please fetch the content of https://example.com
```

The source code for this example can be found in [`src/lib.rs`](src/lib.rs).
