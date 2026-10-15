# Changelog

## 0.4.0

### Rebuilt for the `swc_ecma_ast 29` host ABI

`0.3.0` was built against `swc_core 76` / `swc_ecma_ast 28`. Next.js ≥ 16.4.0 and
`@swc/core` ≥ 1.16.0 moved to `swc_ecma_ast 29`, which changes the CBOR AST wire format, and
they reject the old binary with `Failed to deserialize program received from host`. This
build targets `swc_core 79` / `swc_ecma_ast 29.0.1`.

|                  | Next.js   | `@swc/core` |
| ---------------- | --------- | ----------- |
| `0.4.0` requires | ≥ 16.4.0  | ≥ 1.16.0    |
| `0.3.0` requires | ≤ 16.3.8  | 1.15.x      |

**The match is required in both directions.** Verified for this release: Next.js 16.4.0 and
`@swc/core` 1.16.18 accept `0.4.0` and reject `0.3.0`; Next.js 16.3.0, 16.3.8 and
`@swc/core` 1.15.47 accept `0.3.0` and reject `0.4.0`. So upgrading the bundler past those
lines means upgrading this plugin with it — and staying below them means staying on `0.3.0`,
which is not deprecated.

A trivial file can still transform cleanly under a mismatched pair, so a one-element smoke
test does not prove a pair works; the failure shows up on real input.

If you reach SWC through `@vitejs/plugin-react-swc`, `@swc/jest` or another wrapper, note
that those depend on `@swc/core` by range, so an existing lockfile can keep resolving
1.15.x and fail even after you upgrade this plugin. Force the transitive bump:

```bash
pnpm update @swc/core --depth Infinity   # npm/yarn: delete the lockfile entry and reinstall
```

No behavioural change — ids, options and output are identical. 176 fixtures unchanged.

### `scripts/verify-runtime.cjs` now gates every publish

`cargo test` passes natively without ever crossing the plugin FFI boundary, and `cargo build`
proves only that the wasm compiles. Both were green for a `plugin.wasm` no current host could
load. The runtime check now runs from `prepublishOnly`, so it covers `release.sh` and both
workflows, and as its own CI job on pull requests.

## 0.3.0

### The iteration forms that still shared one id now get unique ids

Three shapes were previously documented as producing the same static id for every item.
None of them needs configuration:

| Source | Before | Now |
| ------ | ------ | --- |
| `R.map((item) => <li/>, items)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `items.map(renderItem)` | `renderItem.li` on every item | `renderItem.li-0`, `renderItem.li-1`, … |
| `items.map((item, [i]) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |

- **A call-site counter** covers callees that pass no index at all. Where the callback is
  argument 0 and a collection argument follows it, the appended index parameter takes a
  default that reads a counter bound just outside the call
  (`((c) => R.map((item, i = c++) => …, items))(0)`). A default is evaluated per
  invocation and only when nothing was passed, so the binding always holds a number — and
  the emitted segment needs no guard. The counter resets each time the call runs. A
  curried `map(fn)` is excluded: it evaluates once and may be stored, where a counter
  would drift. Opt out with `indexCounter: false`.
- **Callbacks passed by reference** are indexed at their declaration. The names handed to
  map-like calls are collected in a pass of their own first, so the declaration may appear
  before the call, after it, or hoisted. A declaration in another file is still out of
  reach. Opt out with `resolveCallbackRefs: false`.
- **A destructuring pattern in the index slot** contributes the first identifier it binds,
  guarded like any other inference. `(item, { index }) => …` and `(item, [i]) => …` both
  work; a pattern that binds nothing still yields a static id.

An index **you** declare is still used exactly as written, so
`R.map((item, i) => …, items)` — a declared index on a callee that passes none — remains
the one shape that can produce `List.li-undefined`. Let the plugin infer it instead.

### `instanceIds` — ids per instance rather than per component (opt-in)

A component rendered twice repeated its ids. With `instanceIds: true` each component
prefixes its ids with the attribute value its parent passed down — the attribute the
plugin already injects on the child element — so instances become distinct:

```jsx
<Badge/> <Badge/>            // Toolbar.Badge.span, Toolbar.Badge2.span
{items.map(() => <Badge/>)}  // Rows.Badge-0.span, Rows.Badge-1.span, …
```

A component that receives no prefix falls back to its own name and keeps exactly the id it
had before. **Off by default**: it adds a path prefix to every id below the top level, so
existing selectors for nested components must be updated before turning it on.

### Also

- `scripts/verify-runtime.cjs` runs the built `plugin.wasm` through a real SWC host and
  asserts the ids of *rendered* markup, once as emitted and once minified with mangling —
  the two things fixtures cannot check.
- The readme's claim that an id can never be `-undefined` was overstated; it holds for
  every index the plugin infers, not for one the source declares.

## 0.2.0

### Unique ids for every iteration form

Elements rendered inside a map callback now get a unique runtime id per item wherever an
index can be reached. The plugin takes the first of these that is available:

| Source | Index comes from |
| ------ | ---------------- |
| `items.map((item, i) => …)` | the declared parameter |
| `items.map((item, i = 0) => …)` | the parameter behind the default |
| `items.map((...args) => …)` | `args[1]` — the signature is left untouched |
| `items.map((item, ...rest) => …)` | `rest[0]` |
| `items.map((item) => …)` | an appended `__testidIdx0` parameter |

and recognises the call shape in every position a callback can occupy:

```jsx
{items.flatMap((item) => <li/>)}       // flatMap, like map
{_.map(items, (item) => <li/>)}        // lodash, Children.map, $.map
{items.map((item) => <li/>, this)}     // .map(cb, thisArg) is still Array.prototype.map
{items$.pipe(map((item) => <li/>))}    // the RxJS operator passes (value, index)
```

Nested callbacks now contribute one id segment each — `Table.td-0-0`, `Table.td-0-1`,
`Table.td-1-0` — where previously only the innermost index was used, so every row
produced the same cell ids.

Where a callee turns out not to pass an index — curried ramda, lodash-fp,
`R.map(fn, list)`, or a helper of your own named `map` — the segment drops out and the id
is the plain static one. Inference is therefore attempted broadly: a wrong guess costs
nothing, while a missed one costs a duplicate id.

### Breaking: some ids change

Only ids that were previously **duplicated** are affected. Update any selector that
targets them:

| Source | 0.1.x | 0.2.0 |
| ------ | ----- | ----- |
| `items.map((item) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `items.flatMap((item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `_.map(items, (item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `rows.map((r, i) => <tr>{r.map((c, j) => <td/>)}</tr>)` | `Table.td-0`, repeated per row | `Table.td-0-0`, `Table.td-0-1`, … |

Each row has its own opt-out, so a large codebase can migrate in stages: row 1
`autoIndex: false`, row 2 `mapMethods: ["map"]`, row 3 `collectionFirstMap: false`,
row 4 `nestedIndexes: false`. Setting all four reproduces the 0.1.x scheme exactly. To pin
an individual id permanently, set the attribute yourself — the plugin never overwrites one
that is already present.

### New options

All default to the behaviour that produces the most unique ids; they exist to turn
inference off, not to switch it on.

- **`autoIndex`** (`boolean`, default `true`) — append an index parameter to map
  callbacks that declare none.
- **`mapMethods`** (`string[]`, default `["map", "flatMap"]`) — callee names treated as
  iteration. Narrow it to leave a name alone, or widen it to add your own iterator.
- **`collectionFirstMap`** (`boolean`, default `true`) — recognise
  `f(collection, callback)`.
- **`nestedIndexes`** (`boolean`, default `true`) — one id segment per enclosing
  callback. Turning it off makes nested ids collide across outer iterations.

### Notes

- An appended index is emitted guarded —
  `{"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}` — so a helper of
  your own named `map` that does not pass an index degrades to the plain static id rather
  than rendering `List.li-undefined`.
- Callbacks that render no JSX keep their original signature, and nothing is appended to a
  callback passed by reference or one whose parameter slots are already full.
- Appending a parameter changes the callback's `Function.length`. Array methods, lodash
  and `React.Children.map` do not branch on callback arity; if something you call does,
  set `autoIndex: false`. Rest-parameter callbacks are read from, never appended to, so
  their arity is unchanged.
- Ids remain per component, not per instance: a component rendered twice repeats its ids.
- A plugin config that cannot be deserialized now **fails the build with an explanatory
  error** instead of being silently ignored. Deserialization is all-or-nothing, so one
  mistyped value (`"autoIndex": "false"`) previously discarded every other option too.

## 0.1.1 – 0.1.2

Static `Component.element` ids, with dynamic ids only for map callbacks that already
declared an index parameter. (0.1.3 was prepared but never published.)
