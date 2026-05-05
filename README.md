# fledge-plugin-test-metadata

WASM test plugin for the [fledge](https://github.com/CorvidLabs/fledge) `metadata` capability.

## What it tests

Verifies that WASM plugins can query project metadata through the `fledge::metadata` host import when granted `metadata = true` in `plugin.toml`. Runs the following test cases:

- **fledge_config** -- reads the project's fledge configuration
- **git_status** -- retrieves current git working tree status
- **git_tags** -- lists git tags in the repository
- **git_log** -- retrieves recent commit history
- **Environment (filtered)** -- reads environment variables while confirming sensitive tokens (`GITHUB_TOKEN`, `GH_TOKEN`, `ANTHROPIC_API_KEY`) are filtered out
- **Multiple keys in one request** -- fetches all four metadata keys in a single call
- **Unknown key** -- requesting an unknown key is handled gracefully without crashing
- **Empty key list** -- an empty request returns a valid response
- **Negative tests** -- filesystem, network, and process spawn are all blocked (only `metadata` is granted)

## Capability exercised

```toml
[capabilities]
exec = false
store = false
metadata = true
filesystem = "none"
network = false
```

## Install and run

```bash
fledge plugins install corvid-agent/fledge-plugin-test-metadata
fledge plugins run test-metadata
```

## Build from source

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

The compiled WASM binary is written to `target/wasm32-wasip1/release/test-metadata.wasm`.
