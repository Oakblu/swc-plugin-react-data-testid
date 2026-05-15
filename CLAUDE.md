# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Does

A dual-compiler plugin suite that automatically injects `data-testid` attributes into React JSX elements at build time. Two independent implementations share the same behavior:

- **Babel plugin** (TypeScript) — root package, published as `babel-plugin-react-data-testid-generator`
- **SWC plugin** (Rust/WASM) — `swc-plugin-react-data-testid/`, published as `swc-plugin-react-data-testid`

## Commands

### Babel Plugin (root)

```bash
yarn build              # Compile TypeScript to dist/
yarn test               # Run Jest suite
yarn test -- --testNamePattern="name"   # Run specific test by name
yarn test src/__tests__/index.test.ts   # Run specific file
yarn test:coverage      # Run with coverage (80% threshold enforced)
yarn test:update-snapshots              # Regenerate snapshots
yarn lint               # ESLint
yarn fmt                # Prettier format
yarn fmt:check          # Check formatting
yarn tsc --noEmit       # Type check
```

### SWC Plugin (`swc-plugin-react-data-testid/`)

```bash
cargo test                                            # Run Rust tests
UPDATE=1 cargo test                                   # Regenerate fixture outputs
cargo build --target wasm32-wasip1 --release          # Build WASM binary
npm run build                                         # Build WASM and copy to plugin.wasm
```

### Full CI Check Sequence (Babel plugin)

```bash
yarn lint && yarn tsc --noEmit && yarn fmt:check && yarn test:coverage && yarn build
```

## Architecture

### Core ID Generation Logic

Both plugins implement the same algorithm:

1. Detect component entry points: `FunctionDeclaration`, `ArrowFunctionExpression`, `FunctionExpression`, and `ClassMethod` named `render`
2. Extract the component name from the declaration or variable name
3. Traverse all JSX elements within the component body
4. For each JSX element, derive a unique ID using a per-component counter map:
   - First `<button>` → `ComponentName.button`
   - Second `<button>` → `ComponentName.button2`
   - Member expressions: `<Modal.Header>` → `ComponentName.Modal.Header`
5. Skip elements that already have the target attribute

The scoped counter map (`component_counters` in Rust, `componentCounters` in TS) is reset per component, so IDs are unique within a component but not globally.

### Babel Plugin (`src/index.ts`)

Standard Babel plugin using the visitor pattern (`PluginObj<PluginState>`). Key functions: `getUniqueElementId`, `addDataTestIds`, `processJSXElements`, `getComponentName`.

### SWC Plugin (`swc-plugin-react-data-testid/src/lib.rs`)

Rust implementation using SWC's `VisitMut` trait, compiled to WASM via `wasm32-wasip1`. Exposes a `#[plugin_transform]` entry point. Configuration deserialized from JSON via `TransformPluginProgramMetadata`.

**SWC version pinning is critical**: SWC plugins must be compiled against the exact same SWC version used by the consumer (Next.js, @swc/core, etc.). The current pins in `Cargo.toml` target Next.js 16.2.6 / @swc/core 1.10.18. Changing `swc_core`, `swc_ecma_ast`, or `swc_plugin_proxy` versions requires coordinating with downstream consumers.

### Test Structure

- **Babel**: Jest + `babel-plugin-tester` + `ts-jest`. Tests in `src/__tests__/` use snapshot testing and inline assertions.
- **SWC**: Rust fixture tests in `tests/fixture.rs`. Each fixture is a directory with `input.tsx` and `output.tsx`. Run `UPDATE=1 cargo test` to regenerate outputs.

### Examples

- `example/babelrc/` — Next.js 14 with Babel plugin via `.babelrc.json`
- `example/vite/` — Vite 4 + React 18 with Babel plugin via `vite.config.js`
- `example/nextjs-swc/` — Next.js 16 with SWC plugin via `next.config.js` `experimental.swcPlugins`
