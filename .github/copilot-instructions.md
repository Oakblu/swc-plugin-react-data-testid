# Copilot / AI assistant instructions

This repository has two AI-facing documents, and picking the wrong one wastes your time:

- **Editing this repository** (changing the plugin, adding fixtures, releasing) →
  read [`CLAUDE.md`](../CLAUDE.md). It holds the build invariants, all of which exist
  because each was violated silently in the past and none are caught by `cargo test`.
- **Using the published plugin** (predicting generated ids, writing selectors,
  configuring options) → read [`AGENTS.md`](../AGENTS.md).

## Before reporting any change complete

```bash
./scripts/check-invariants.sh && cargo test && cargo build --target wasm32-wasip1 --release
```

If `check-invariants.sh` exits non-zero, re-run it with `--fix`, then fix by hand what
`--fix` could not. Do not report work as done while it is failing.

## The traps that bite hardest

- **`cargo test` alone is not sufficient.** Native tests never cross the plugin FFI
  boundary, so a dependency-pin mismatch passes the test suite and fails only at the
  WASM link step with undefined host symbols. Always run the WASM build too.
- **A new fixture directory does not run until you touch `tests/fixture.rs`.**
  `#[testing::fixture]` expands its glob at *compile* time, so cargo reuses the cached
  binary and the suite still reports green. Run
  `touch tests/fixture.rs && UPDATE=1 cargo test`.
- **`swc_core`, `swc_ecma_ast` and `swc_plugin_proxy` versions are pinned together.**
  Changing one requires coordinating with downstream SWC consumers.
- **Never swallow a `Result`** in `src/lib.rs` (`let _ = …`, `.ok()` on config parsing).
  The plugin runs in a WASI sandbox where filesystem calls always fail, so a swallowed
  error becomes a silent no-op — and serde deserialization is all-or-nothing, so one
  mistyped option silently discards every option the user set.
- **Do not publish or commit without an explicit request.**

Full detail, including the reasoning behind each rule, is in `CLAUDE.md`.
