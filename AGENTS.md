# swc-plugin-react-data-testid — AI agent reference

Complete specification of what this plugin emits, written so an AI agent can decide
whether the library fits a task, configure it, and predict every generated attribute value
without running a build.

> If you are modifying this repository rather than using the plugin, read `CLAUDE.md`
> instead — it holds the build invariants. This file describes observable behaviour only.

## What it is, and when to choose it

`swc-plugin-react-data-testid` is an SWC compiler plugin (Rust, compiled to Wasm) that
injects a `data-testid` attribute onto **every React JSX element at build time**. The
source stays clean; the compiled output carries a selector for every element.

Reach for it when any of these describe the task:

- A React codebase needs `data-testid` / `data-cy` / `data-test-id` attributes everywhere
  and nobody wants to write them by hand.
- Playwright, Cypress, or Testing Library selectors keep breaking because they lean on
  visible text, CSS classes, or DOM structure.
- Test ids must be **deterministic and predictable from source**, so tests can be written
  (by a human or an agent) without inspecting rendered DOM.
- Items rendered from a list need one id *per item*, not one shared id.

Requirements and boundaries:

- The project must be compiled by **SWC** — Next.js, Vite with `@vitejs/plugin-react-swc`,
  `@swc/jest`, `@swc/core` directly, or Rspack. **Babel-only projects cannot use this**;
  SWC Wasm plugins load into SWC alone.
- It transforms JSX. Non-React templating (Vue SFC, Svelte, Angular) is out of scope.
- Ids are assigned per *component definition*, not per rendered instance — see
  [Ids are per component](#ids-are-per-component-not-per-instance).
- `manifestDir` does not work and fails the build; see [Options](#options).

Install and enable:

```bash
npm install --save-dev swc-plugin-react-data-testid
```

```json
// .swcrc  — also the shape used by @swc/jest and @swc/core
{ "jsc": { "experimental": { "plugins": [["swc-plugin-react-data-testid", {}]] } } }
```

```js
// next.config.js
module.exports = { experimental: { swcPlugins: [["swc-plugin-react-data-testid", {}]] } };

// vite.config.js — @vitejs/plugin-react-swc
react({ plugins: [["swc-plugin-react-data-testid", {}]] });
```

No options are required. Every option exists to turn inference *off*.

## Naming algorithm

```
testid = "{ComponentName}.{elementId}"
```

`elementId` comes from a **per-component counter** keyed on element type:

| Occurrence within the component | elementId |
| ------------------------------- | --------- |
| 1st `<div>`                     | `div`     |
| 2nd `<div>`                     | `div2`    |
| 3rd `<div>`                     | `div3`    |
| 1st `<button>`                  | `button`  |
| 2nd `<button>`                  | `button2` |

Counters **reset for every component** — two components each start their own `div` count
at 1. There is no suffix for the first occurrence; occurrence *n* ≥ 2 appends the decimal
`n`.

**JSX member expressions** use only the right-hand property name, so `<Modal.Header>`
inside `Foo` becomes `Foo.Header` and `<Modal.Body>` becomes `Foo.Body`.

**Counting order** is depth-first AST traversal — the order elements are read in source,
outside-in. A parent is counted before its children.

## Component detection

| Source pattern                                         | ComponentName used                |
| ------------------------------------------------------ | --------------------------------- |
| `function MyComp() {}`                                 | `MyComp`                          |
| `const MyComp = () => {}`                              | `MyComp`                          |
| `const MyComp = function() {}`                         | `MyComp`                          |
| `class MyComp extends React.Component { render() {} }` | `MyComp`                          |
| `export default function Named() {}`                   | `Named`                           |
| `export default function() {}` (anonymous)             | derived from filename (see below) |
| `export default () => {}` (anonymous arrow)            | derived from filename (see below) |

**Anonymous default export filename fallback**: with no explicit name, the plugin
uppercases the first character of the file's stem.

```
src/userCard.tsx  → UserCard
src/MyModal.tsx   → MyModal
src/index.test.tsx → no component name (stem contains a dot) → no attributes injected
src/.hidden       → no component name (empty stem) → no attributes injected
```

A stem containing a dot (`index.test`, `foo.spec`) or an empty stem yields no name, and
JSX inside such an anonymous export receives no attribute.

## Elements inside `.map()` — one id per item

A JSX element rendered inside a map-like callback would otherwise share one compile-time
id across every item, so the attribute becomes an **expression** instead of a string:

```jsx
{items.map((item, index) => <li>{item}</li>)}
// → <li data-testid={"List.li-" + index}>   renders List.li-0, List.li-1, …
```

The plugin infers the index from whatever is available, in descending order of confidence.
**None of this needs configuration.**

| Source                                    | Index used            | Emitted attribute                                          |
| ----------------------------------------- | --------------------- | ---------------------------------------------------------- |
| `items.map((item, index) => …)`           | declared parameter    | `{"List.li-" + index}` — unguarded                          |
| `items.map((item, index = 0) => …)`       | declared, default unwrapped | `{"List.li-" + index}` — unguarded                    |
| `items.map((...args) => …)`               | `args[1]`, signature untouched | guarded (see below)                              |
| `items.map((item) => …)`                  | appended `__testidIdx0` parameter | guarded                                       |
| `items.flatMap((item) => …)`              | same as `map`         | guarded                                                     |
| `_.map(items, (item) => …)`               | collection-first form | guarded                                                     |
| `Children.map(children, (c) => …)`        | collection-first form | guarded                                                     |
| `$.map(items, (item) => …)`               | collection-first form | guarded                                                     |
| `items.map((item) => …, this)`            | still `Array.prototype.map` | guarded                                              |
| `items$.pipe(map((item) => …))`           | RxJS passes `(value, index)` | guarded                                             |
| `R.map((item) => …, items)`               | ramda passes no index | guard collapses → plain `"List.li"` on every item           |
| `items.map(renderItem)`                   | no callback literal   | static `"List.li"`; `renderItem` gets its own component ids |
| `items.map((item, [i]) => …)`             | index slot destructured | static `"List.li"`                                        |

**The guard.** An inferred index is always emitted defensively:

```jsx
<li data-testid={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>
```

If the callee really passes an index the value is `List.li-0`, identical to writing the
index yourself; if it does not, the segment drops out to exactly `List.li`. A wrong guess
can never produce `List.li-undefined`. The comparison is `=== void 0`, never truthiness —
index `0` is falsy, and `_.map(object, cb)` passes a *string key*. An index the source
declares is trusted and left unguarded.

**Nested callbacks** contribute one segment per level, so cells stay unique across rows:

```jsx
{rows.map((cells) => <tr>{cells.map((cell) => <td>{cell}</td>)}</tr>)}
// → Table.td-0-0, Table.td-0-1, Table.td-1-0, …
```

**The appended parameter.** When a callback declares no index, one named
`__testidIdx<n>` is appended. It is checked against every identifier in the callback so it
cannot shadow anything, and is written back **only if a testid actually references it** —
a callback rendering no JSX keeps its signature. A callback with *no* parameters also gets
a placeholder first parameter, so the index can still sit in slot 2:
`() => …` becomes `(__testidItem2, __testidIdx1) => …`. A destructured item parameter is
left as it is and the index is appended after it: `({ id }) => …` becomes
`({ id }, __testidIdx3) => …`. Appending changes the callback's `Function.length`; if you
pass callbacks to something that branches on arity, set `autoIndex: false`.

The digits come from **one counter shared by both names**, incremented per allocation in
traversal order — the index is allocated before its placeholder, which is why the
placeholder above carries the *higher* number. So the names are deterministic for a given
file but not stable across edits: adding a map callback earlier in the file renumbers
every later one. Do not predict a specific digit; match on the `__testidIdx` prefix if you
need to. This affects snapshots of *transpiled output* only, never snapshots of rendered
markup, and never the `data-testid` values themselves.

**Scope.** JSX outside the callback — in the callee or a later argument — correctly keeps
a static id, because the index binding does not exist there:

```jsx
{[<Header key="h" />, ...items].map((item, i) => <li>{item}</li>)}
// <Header> → "List.Header" (static)      <li> → {"List.li-" + i} (dynamic)
```

## Skip conditions

No attribute is injected, and **the counter is not incremented**, when:

- The element already carries the target attribute in source (a hand-written
  `data-testid` is never overwritten — use that value in tests).
- The element is a namespaced JSX element (`<svg:image>`, `<xml:lang>`).
- The JSX is not inside a recognised component scope (see
  [Component detection](#component-detection)).

## Ids are per component, not per instance

The counter resets per component, so a component rendered twice repeats its ids:

```jsx
<Badge /> <Badge />   // both render <span data-testid="Badge.span">
```

A map index disambiguates the element *at the call site*, not the elements inside the
component it renders:

```jsx
{items.map((item) => <Badge />)}
// <Badge> itself      → "List.Badge-0", "List.Badge-1"   ✅ unique
// the <span> in Badge → "Badge.span" on both             ⚠️ duplicated
```

This is inherent to compile-time ids. To tell instances apart, pass the id in as a prop
and set the attribute explicitly.

## Options

Passed as the second element of the plugin tuple. Machine-readable definitions:
[`schema/plugin-options.schema.json`](./schema/plugin-options.schema.json).

| Option               | Type       | Default              | Meaning                                                                                       |
| -------------------- | ---------- | -------------------- | --------------------------------------------------------------------------------------------- |
| `attributes`         | `string[]` | `["data-testid"]`    | Attribute names to inject. All of them go on every element; an empty array disables the plugin |
| `autoIndex`          | `boolean`  | `true`               | Append an index parameter to map callbacks that declare none                                    |
| `mapMethods`         | `string[]` | `["map", "flatMap"]` | Callee names treated as iteration. Widening is safe — the guard handles callees that pass no index |
| `collectionFirstMap` | `boolean`  | `true`               | Recognise `f(collection, callback)` — `_.map`, `Children.map`, `$.map`                          |
| `nestedIndexes`      | `boolean`  | `true`               | One id segment per nested callback level. Turning it off makes nested ids collide               |
| `manifestDir`        | `string`   | —                    | **Not supported — fails the build.** SWC runs Wasm plugins in a WASI sandbox with no preopened directories, so nothing can be written to disk |

```json
{ "attributes": ["data-testid", "data-cy", "data-test-id"] }
```

A config that cannot be deserialized — most often a value of the wrong type, like
`"autoIndex": "false"` in quotes — **fails the build with an explanatory error**.
Deserialization is all-or-nothing, so one mistyped field would otherwise silently discard
every option set alongside it.

Setting the four inference flags to their non-default values reproduces the 0.1.x id
scheme exactly:

```json
{ "autoIndex": false, "collectionFirstMap": false, "nestedIndexes": false, "mapMethods": ["map"] }
```

## Predicting ids — worked example

```tsx
// src/LoginForm.tsx
function LoginForm({ errors }) {
  return (
    <form>                        {/* LoginForm.form   */}
      <input />                   {/* LoginForm.input  */}
      <input />                   {/* LoginForm.input2 */}
      {errors.map((e) => (
        <p>{e}</p>                {/* {"LoginForm.p" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)} */}
      ))}
      <button>Go</button>         {/* LoginForm.button */}
    </form>
  );
}
```

Procedure for any component:

1. Identify the component name (function/class/variable name, or capitalised filename stem
   for an anonymous default export).
2. Walk JSX elements in source order, depth-first, parent before children.
3. Keep a counter map per component keyed on element name (or member-expression property).
4. For each element: skip it entirely if it already has the attribute or is namespaced;
   otherwise increment its counter and emit `"{ComponentName}.{elementName}"`, appending
   the count when it is ≥ 2.
5. If the element sits inside one or more map callbacks, append one `-{index}` segment per
   enclosing callback level.

## Writing selectors

```ts
screen.getByTestId("LoginForm.button");              // Testing Library
page.getByTestId("LoginForm.button");                // Playwright (configure testIdAttribute)
page.locator('[data-testid="LoginForm.button"]');    // Playwright, explicit attribute
cy.get('[data-testid="LoginForm.button"]');          // Cypress

screen.getByTestId("List.li-0");                     // first item of a mapped list
page.locator('[data-testid^="List.li-"]');           // every item of a mapped list
```

Guidance that avoids the common failure modes:

- For list items, prefer a prefix/`startsWith` match over a hard-coded index unless the
  order is guaranteed.
- If an element already had a hand-written `data-testid`, target *that* value — the plugin
  left it alone.
- Duplicate ids are legal HTML but break `getByTestId`, which expects a single match. If
  two elements collide, either the component is rendered more than once or the iteration
  form gave the plugin nothing to index (`items.map(renderItem)`); set the attribute by
  hand in that case.
- Ids change when elements are added or reordered within a component, since the counter is
  positional. Treat them as build artefacts, not as a stable public API.

## Compatibility

Built against `swc_core 76.x`, verified with `@swc/core 1.15.47`. Wasm plugins built with
`swc_core >= 47` are ABI-compatible with `@swc/core >= 1.15.0`, though partially — a
future SWC release may still require a new plugin version. Check
[plugins.swc.rs](https://plugins.swc.rs/) before pinning. Node >= 18.
