# fledge-plugin-test-metadata

WASM plugin that tests the `metadata` capability for [fledge](https://github.com/CorvidLabs/fledge).

Verifies that WASM plugins can query project metadata (fledge config, git status/tags/log, filtered env) via `fledge::metadata` when granted `metadata = true`, and that other capabilities remain blocked.

## Install & Run

```bash
fledge plugins install CorvidLabs/fledge-plugin-test-metadata
fledge plugins run test-metadata
```

## Requirements

- [fledge](https://github.com/CorvidLabs/fledge) with WASM runtime support
- `wasm32-wasip1` Rust target: `rustup target add wasm32-wasip1`
