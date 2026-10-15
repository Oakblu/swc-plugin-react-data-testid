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
./scripts/check-invariants.sh && cargo test && npm run build && node scripts/verify-runtime.cjs
```

`scripts/verify-runtime.cjs` drives the built `plugin.wasm` through a real `@swc/core`
host and asserts the ids of *rendered* markup, once as emitted and once minified with
mangling — the three things fixtures cannot do (cross the FFI boundary, minify, run). It
borrows `@swc/core` and `react-dom` from `example/jest`, so that example has to be
installed, and it reads `plugin.wasm` by absolute path, so run `npm run build` first.

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
| 3 | No `let _ = std::fs::…` and no `.ok()` on config parsing in `src/lib.rs` | Swallowed errors turn a broken feature into a silent no-op. `serde` deserialization is all-or-nothing, so one mistyped option silently discarded *every* option until this was reported | manual |
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

12. **Infer an index wherever one plausibly exists; never require configuration to get
    unique ids.** Detection is deliberately broad — any callee named in `mapMethods` with
    a callback in argument 0 or 1 — because the syntax cannot distinguish callees that
    pass an index from ones that do not (`map(fn)` is both RxJS and curried ramda).
    Narrowing detection guarantees duplicate ids for everything it excludes, while
    guessing wrongly costs nothing once the segment is guarded (rule 14). Prefer an index
    already in scope over one that has to be introduced: declared parameter → rest element
    (`args[1]`, no signature change) → identifier bound by a pattern in the index slot →
    appended parameter (with a counter default where one is available, rule 15). A
    callback passed by *reference* is indexed at its declaration in the same file, which
    is why names are collected in a pass before anything is rewritten — the declaration
    may be hoisted or come later. Options may only ever *remove* inference; a default that
    has to be switched on is a bug, with exactly one exception: `instanceIds` changes ids
    that already work, so it must be opted into. New shapes need a fixture in
    `tests/fixture/map-uncertain-callees/` (inferred + guarded),
    `tests/fixture/map-index-counter/` (counter-eligible),
    `tests/fixture/map-callback-refs/` (by reference), or
    `tests/fixture/map-no-inject-cases/` (nothing to work with).

13. **Every generated binding name must be collision-checked against the callback it is
    inserted into** (`fresh_ident`), and carry the parameter scope's `SyntaxContext`.
    A name that already appears in the body would shadow it; the per-file counter is
    what keeps nested callbacks from generating the same name twice, since scanning an
    inner callback cannot yet see an outer generated name.

14. **Any index the plugin inferred must be emitted guarded unless it is bound by
    construction; a declared one must not be guarded at all.**
    `{"List.li" + (i === void 0 ? "" : "-" + i)}` degrades to the plain static id when the
    callee turns out not to pass an index. This guard is what licenses the broad detection
    in rule 12 — without it, every widening would risk `List.li-undefined` on every item.
    Guard appended parameters *and* rest-element reads (`args[1]` may not exist). Compare
    with `=== void 0`, never a truthiness or numeric test: index `0` is falsy, and
    `_.map(object, cb)` passes a string key — both verified against real lodash in
    `example/vitest/src/__tests__/MapHosts.test.jsx`. Two sources are exempt. Indices the
    source declares stay unguarded: writing `(item, i) =>` asserts the binding, and
    guarding it would bloat the common output for nothing. So does an index whose
    parameter default reads a call-site counter (rule 15) — the default runs precisely
    when the callee passed nothing, so the binding always holds a number.

15. **The call-site counter may only be used where the call iterates immediately.**
    `((c) => R.map((item, i = c++) => …, items))(0)` gives a real 0..n-1 index to a callee
    that passes none. It requires the callback in argument 0 *and* a further argument: a
    curried `map(fn)` evaluates once and the function it returns may be stored and reused,
    so the counter would climb past the item positions instead of tracking them. It is
    also refused when the call contains `await` or `yield` outside a nested function,
    because the counter binding is introduced by an arrow wrapped around the call and an
    arrow can do neither. Both exclusions fall back to the guarded appended index.

16. **An instance prefix must never be read from a parameter that is not props.** With
    `instanceIds`, a component reads the prefix from its first parameter — but a function
    reached through `items.map(row)` receives the *item* there, so it takes an index and
    never a prefix. A class reads `this.props`, which is not the instance inside a
    `function` expression, so the prefix is suppressed there. An appended props parameter
    carries `= {}`: destructuring `undefined` throws, and a component that declared no
    parameter can legitimately be called with no argument.

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
  `_.map(items, cb)` → arg 1) for any callee named in `mapMethods`. It does *not* try to
  decide whether that callee passes an index — it cannot (rule 12).
- `IndexSource` ranks where the index comes from: `Declared` (a named parameter, trusted
  and unguarded), `RestElement` (`args[1]` / `rest[0]` — no signature change),
  `Destructured` (the first identifier a pattern in the index slot binds — the slot is
  occupied so nothing can be appended), then `Appended` / `Counted` (`fresh_ident`,
  rule 13, written back **only if a testid referenced it**, so callbacks without JSX keep
  their signature). Everything but `Declared` and `Counted` is guarded (rule 14).
- `declared_index_ident_from` accepts any second parameter that binds one identifier,
  unwrapping a default (`(item, index = 0) =>`). A destructuring pattern falls through to
  `destructured_index_ident_from`, which must be tried *after* `rest_index_source_from`:
  a rest element at that position holds the argument list, not the index.
- `map_indices` is a stack: an element inside nested callbacks gets one segment per
  level (`{"Table.td-" + r + "-" + c}`), otherwise the inner index would repeat for
  every outer iteration. The stack is only active while visiting the callback argument,
  never the callee or the other arguments, where the binding does not exist.
- `counter_call_site` decides whether a `Counted` index is available (rule 15); when one
  is used, the whole call is replaced by `((seq) => call)(0)` *after* its arguments have
  been visited, and only if a testid referenced the index.
- `CallbackRefScan` runs over the program in `visit_mut_program` and collects the names
  handed to map-like calls by reference. `enter_component_scope` turns a matching
  declaration into a callback: it puts an index in scope for the body and force-guards it,
  because a named function can also be called with no index at all.
- `instance_prefix` holds the prefix in force for the component being visited
  (`instanceIds` only), and `fn_expr_depth` tracks the function expressions entered since
  the component started, because `this.props` is unreadable inside one.

### SWC Plugin (`src/lib.rs`)

Rust implementation using SWC's `VisitMut` trait, compiled to WASM via `wasm32-wasip1`. Exposes a `#[plugin_transform]` entry point. Configuration deserialized from JSON via `TransformPluginProgramMetadata`.

**SWC version pinning is critical**: the plugin's `swc_ecma_ast` major must match the one the consumer's SWC uses (Next.js, @vitejs/plugin-react-swc, @swc/jest, etc.). The current pins in `Cargo.toml` target `swc_core 79.x` / `swc_ecma_ast 29.0.1` — Next.js >= 16.4.0 or `@swc/core` >= 1.16.0. `0.3.0` and earlier (`swc_core 76` / `swc_ecma_ast 28`) serve Next.js <= 16.3.8 / `@swc/core` 1.15.x. There is **no** ABI compatibility to lean on in *either* direction: the major sets the CBOR AST wire format, so an ast-28 plugin fails on an ast-29 host exactly as the reverse, with `Failed to deserialize program received from host`. A trivial file can still transform cleanly under a mismatched pair — verified — so a one-element smoke test proves nothing; only substantial source does. Changing `swc_core`, `swc_ecma_ast`, or `swc_plugin_proxy` versions requires coordinating with downstream consumers, and updating the version table in `readme.md` and `CHANGELOG.md`.

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

- `example/nextjs-swc/` — Next.js 16.4 with SWC plugin via `next.config.js` `experimental.swcPlugins`. Next bundles its own SWC, so the host ABI is pinned by the `next` version here; the other examples float on `@swc/core` and need `pnpm update @swc/core --depth Infinity` to follow a pin change
- `example/jest/` — Jest with `@swc/jest` transformer; run `pnpm test` to verify the plugin injects testids at build time
