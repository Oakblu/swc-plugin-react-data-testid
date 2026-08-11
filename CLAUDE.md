# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Does

An SWC plugin (Rust/WASM) that automatically injects `data-testid` attributes into React JSX elements at build time. Published as `swc-plugin-react-data-testid` on npm.

## Commands

```bash
cargo test                                            # Run Rust tests
UPDATE=1 cargo test                                   # Regenerate fixture outputs
cargo build --target wasm32-wasip1 --release          # Build WASM binary
npm run build                                         # Build WASM and copy to plugin.wasm
./scripts/check-invariants.sh                         # Check repo invariants (see Rules)
./scripts/check-invariants.sh --fix                   # Auto-fix what is safely fixable
```

### Full CI Check

```bash
./scripts/check-invariants.sh && cargo test && cargo build --target wasm32-wasip1 --release
```

## Rules — always check, fix automatically

**Run `./scripts/check-invariants.sh` before reporting any task complete, and again
before preparing a release.** If it exits non-zero, run it with `--fix` and then fix by
hand whatever `--fix` could not. Do not report work as done while it is failing.

These rules exist because each one was violated silently in the past — none of them are
caught by `cargo test`.

| # | Rule | Why it exists | Auto-fix |
| - | ---- | ------------- | -------- |
| 1 | `Cargo.toml` version must equal `package.json` version | They drifted to 0.1.0 vs 0.1.3 because `release.sh` only bumped `package.json` | `--fix` |
| 2 | No untracked files under `tests/` | A regression fixture was left untracked and would not have protected anyone | `--fix` |
| 3 | No `let _ = std::fs::…` in `src/lib.rs` | Swallowed I/O errors turned a broken feature into a silent no-op for users | manual |
| 4 | `plugin.wasm` must be newer than `src/lib.rs` and `Cargo.toml` | Examples silently tested a stale binary, so a fix appeared not to work | `--fix` |
| 5 | Every `tests/**/input.tsx` has a sibling `output.tsx` | `#[testing::fixture]` expands its glob at **compile** time, so a fixture added without touching `tests/fixture.rs` is never run and the suite still reports all green | `--fix` |
| 6 | `wasm32-wasip1` target installed | Without it the WASM link step cannot run at all | manual (`rustup target add`) |

**After adding a fixture directory, run `touch tests/fixture.rs && UPDATE=1 cargo test`.**
Without the touch, cargo reuses the cached test binary and the new fixture silently does
not exist. The first `UPDATE=1` run may exit non-zero — the assertions that read
`output.tsx` can run before the fixture that writes it — so re-run `cargo test` and judge
by that.

### Additional rules the script cannot check

7. **Always run the WASM build, never just `cargo test`.** Native tests do not cross the
   plugin FFI boundary. A dependency-pin mismatch fails only at the WASM link step with
   undefined host symbols (`__set_transform_result`, `__get_transform_context`), and
   `cargo test` passes anyway. This exact mismatch shipped once.

8. **Verify plugin behavior through a real SWC host under minification**, not just
   fixtures. Fixtures do not minify, so they cannot catch generated identifier
   references that fail to bind — those produce `ReferenceError` only in a production
   build. `example/nextjs-swc` (Turbopack prerender) is the guard; a passing
   `vite build` is *not* sufficient, because a dangling reference still builds cleanly.

9. **When syncing `plugin.wasm` into the examples, copy it into the pnpm store path.**
   pnpm hard-copies `file:../..` dependencies instead of symlinking, so editing the root
   `plugin.wasm` does not affect the examples:
   ```bash
   for d in example/*/; do
     t=$(readlink -f "$d/node_modules/swc-plugin-react-data-testid") && cp plugin.wasm "$t/plugin.wasm"
   done
   ```

10. **Any generated identifier reference must reuse the source binding's `Ident`**
   (cloning `sym` *and* `SyntaxContext`), and must only be emitted where that binding is
   actually in scope. Synthesizing a fresh `Ident`, or leaking a callback parameter into
   the callee/other arguments, produces a reference a minifier will not rename.

11. **Do not publish or commit without an explicit request.** Prepare the release
    (version bump, build, `npm pack --dry-run`) and hand over the publish command.

12. **A parameter may only be appended to a callback the callee really calls with an
    index.** `items.map(cb)` / `_.map(items, cb)` do; `R.map(cb, items)` and a bare
    `map(cb)` (curried ramda / lodash-fp / an RxJS operator) do not — appending there
    yields `"-undefined"` in every id. Any new call shape added to `map_call_site` needs
    a fixture in `tests/fixture/map-no-inject-cases/` proving what is *not* touched.

13. **Every generated binding name must be collision-checked against the callback it is
    inserted into** (`fresh_ident`), and carry the parameter scope's `SyntaxContext`.
    A name that already appears in the body would shadow it; the per-file counter is
    what keeps nested callbacks from generating the same name twice, since scanning an
    inner callback cannot yet see an outer generated name.

## Architecture

### Core ID Generation Logic

1. Detect component entry points: `FunctionDeclaration`, `ArrowFunctionExpression`, `FunctionExpression`, and `ClassMethod` named `render`
2. Extract the component name from the declaration or variable name; fall back to the filename for anonymous default exports
3. Traverse all JSX elements within the component body
4. For each JSX element, derive a unique ID using a per-component counter map:
   - First `<button>` → `ComponentName.button`
   - Second `<button>` → `ComponentName.button2`
   - Member expressions: `<Modal.Header>` → `ComponentName.Modal.Header`
5. Skip elements that already have the target attribute

The scoped counter map (`component_counters`) is reset per component, so IDs are unique within a component but not globally.

### Dynamic IDs inside map callbacks

A JSX element inside a map-like callback would otherwise share one compile-time id
across every item, so its attribute becomes an expression instead:
`data-testid={"List.li-" + index}`.

- `map_call_site` recognises which argument holds the callback (`items.map(cb)` → arg 0;
  `_.map(items, cb)`, `Children.map(children, cb)` → arg 1) and whether appending an
  index parameter is safe there (rule 12).
- If the callback declares no index, `fresh_ident` generates one (rule 13) and it is
  appended **only if a testid actually referenced it**, so callbacks without JSX keep
  their signature.
- `map_indices` is a stack: an element inside nested callbacks gets one segment per
  level (`{"Table.td-" + r + "-" + c}`), otherwise the inner index would repeat for
  every outer iteration. The stack is only active while visiting the callback argument,
  never the callee or the other arguments, where the binding does not exist.

### SWC Plugin (`src/lib.rs`)

Rust implementation using SWC's `VisitMut` trait, compiled to WASM via `wasm32-wasip1`. Exposes a `#[plugin_transform]` entry point. Configuration deserialized from JSON via `TransformPluginProgramMetadata`.

**SWC version pinning is critical**: SWC plugins must be compiled against an `swc_core` version compatible with the consumer's SWC (Next.js, @vitejs/plugin-react-swc, @swc/jest, etc.). The current pins in `Cargo.toml` target `swc_core 76.x` / `@swc/core 1.15.x`. Plugins built with `swc_core >= 47` are ABI-compatible with `@swc/core >= 1.15.0`, but only partially — field removals and type changes can still break it. Changing `swc_core`, `swc_ecma_ast`, or `swc_plugin_proxy` versions requires coordinating with downstream consumers.

`swc_ecma_ast` and `swc_plugin_proxy` are pinned with `=` and must match the versions `swc_core` links against. A mismatch fails the WASM link with undefined host symbols (`__set_transform_result`, `__get_transform_context`). **`cargo test` does not catch this** — native tests never cross the plugin FFI boundary, so always run the WASM build too.

### Test Structure

Rust fixture tests in `tests/fixture.rs`. Each fixture is a directory with `input.tsx` and `output.tsx`. Run `UPDATE=1 cargo test` to regenerate outputs.

Fixture suites:
- `tests/fixture/` — default options
- `tests/fixture-options/` — multiple attributes (data-testid + data-cy)
- `tests/fixture-playwright/` — custom attribute (data-test-id)
- `tests/fixture-multi-attrs/` — four attributes
- `tests/fixture-filename/` — anonymous export filename fallback (reads `filename.txt` per fixture)

### Examples

- `example/nextjs-swc/` — Next.js 16 with SWC plugin via `next.config.js` `experimental.swcPlugins`
- `example/jest/` — Jest with `@swc/jest` transformer; run `pnpm test` to verify the plugin injects testids at build time
