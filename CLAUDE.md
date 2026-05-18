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
```

### Full CI Check

```bash
cargo test && cargo build --target wasm32-wasip1 --release
```

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

### SWC Plugin (`src/lib.rs`)

Rust implementation using SWC's `VisitMut` trait, compiled to WASM via `wasm32-wasip1`. Exposes a `#[plugin_transform]` entry point. Configuration deserialized from JSON via `TransformPluginProgramMetadata`.

**SWC version pinning is critical**: SWC plugins must be compiled against the exact same SWC version used by the consumer (Next.js, @vitejs/plugin-react-swc, @swc/jest, etc.). The current pins in `Cargo.toml` target Next.js 16.2.6 / @swc/core 1.10.18. Changing `swc_core`, `swc_ecma_ast`, or `swc_plugin_proxy` versions requires coordinating with downstream consumers.

### Test Structure

Rust fixture tests in `tests/fixture.rs`. Each fixture is a directory with `input.tsx` and `output.tsx`. Run `UPDATE=1 cargo test` to regenerate outputs.

Fixture suites:
- `tests/fixture/` — default options
- `tests/fixture-options/` — multiple attributes (data-testid + data-cy)
- `tests/fixture-playwright/` — custom attribute (data-test-id)
- `tests/fixture-multi-attrs/` — four attributes
- `tests/fixture-filename/` — anonymous export filename fallback (reads `filename.txt` per fixture)

### Example

- `example/nextjs-swc/` — Next.js 16 with SWC plugin via `next.config.js` `experimental.swcPlugins`
