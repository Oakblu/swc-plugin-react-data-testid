# swc-plugin-react-data-testid

[![npm version](https://img.shields.io/npm/v/swc-plugin-react-data-testid.svg)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![npm downloads](https://img.shields.io/npm/dm/swc-plugin-react-data-testid.svg)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![Build Status](https://github.com/Oakblu/plugin-react-data-testid-generator/workflows/🦀%20SWC%20Plugin%20CI/badge.svg)](https://github.com/Oakblu/plugin-react-data-testid-generator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Automatically adds `data-testid` attributes to your React components during the build process, making it easier to write reliable end-to-end tests. Features **component-scoped unique counters** for predictable, collision-free test IDs.

---

## What it solves

Writing `data-testid` by hand is tedious, gets skipped under deadline, and leaves tests
reaching for CSS classes, DOM structure, or visible text — selectors that break on the
next refactor or copy change. This plugin puts a test id on **every** JSX element at
compile time, so:

- Source files stay free of testing attributes; the compiled output carries them.
- Ids are **deterministic from the source alone** — `LoginForm.button`,
  `LoginForm.input2` — so a selector can be written without inspecting rendered DOM.
- Elements rendered from a list get one id **per item** (`List.li-0`, `List.li-1`), not
  one id shared by all of them.
- Any hand-written `data-testid` already in the source is left untouched.

Reach for it if you are looking for: automatic `data-testid` generation for React, build-time
test id injection, stable Playwright/Cypress/Testing Library selectors, `data-cy` or
`data-test-id` codegen, or a way to stop hand-writing test attributes across a codebase.

**Requirements:** the project must be compiled by SWC — Next.js, Vite via
`@vitejs/plugin-react-swc`, `@swc/jest`, `@swc/core`, or Rspack. SWC Wasm plugins load
into SWC only, so a Babel-only toolchain cannot use this.

**Using an AI assistant?** Point it at [`AGENTS.md`](./AGENTS.md) — the complete
specification of the naming algorithm, precise enough to predict every generated id
without running a build. [`llms.txt`](./llms.txt) indexes the docs, and
[`schema/plugin-options.schema.json`](./schema/plugin-options.schema.json) defines every
option in machine-readable form. All three ship inside the npm package, so they are also
readable from `node_modules/swc-plugin-react-data-testid/` or over the CDN at
`https://unpkg.com/swc-plugin-react-data-testid/AGENTS.md`.

---

## Installation

```bash
npm install --save-dev swc-plugin-react-data-testid
# or
yarn add -D swc-plugin-react-data-testid
```

## Basic Setup

Add the plugin to your `.swcrc`:

```json
{
  "jsc": {
    "experimental": {
      "plugins": [["swc-plugin-react-data-testid", {}]]
    }
  }
}
```

## Framework-Specific Setup

<details>
<summary><strong>Next.js (SWC)</strong></summary>

Configure in `next.config.js`:

```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    swcPlugins: [["swc-plugin-react-data-testid", {}]],
  },
};

module.exports = nextConfig;
```

</details>

<details>
<summary><strong>Vite with @vitejs/plugin-react-swc</strong></summary>

Configure in `vite.config.js`:

```js
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig({
  plugins: [
    react({
      plugins: [["swc-plugin-react-data-testid", {}]],
    }),
  ],
});
```

</details>

<details>
<summary><strong>@swc/jest</strong></summary>

Configure in `jest.config.js`:

```js
module.exports = {
  transform: {
    "^.+\\.[jt]sx?$": [
      "@swc/jest",
      {
        jsc: {
          experimental: {
            plugins: [["swc-plugin-react-data-testid", {}]],
          },
        },
      },
    ],
  },
};
```

</details>

<details>
<summary><strong>Vitest</strong></summary>

Configure in `vite.config.js`:

```js
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig({
  plugins: [
    react({
      plugins: [["swc-plugin-react-data-testid", {}]],
    }),
  ],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.js"],
  },
});
```

Create `vitest.setup.js`:

```js
import "@testing-library/jest-dom/vitest";
```

</details>

### With Custom Attributes

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-react-data-testid",
          { "attributes": ["data-testid", "data-cy"] }
        ]
      ]
    }
  }
}
```

### SWC Version Compatibility

SWC plugins are compiled against a specific `swc_core`, and the plugin's `swc_ecma_ast` major must match the one your bundler's SWC uses. Pick the row matching your bundler — **the match is required in both directions**, so an older plugin on a newer bundler fails exactly like a newer plugin on an older bundler.

| Plugin version      | `swc_core` | `swc_ecma_ast` | Use with                                 |
| ------------------- | ---------- | -------------- | ---------------------------------------- |
| `0.4.0` and later   | 79         | 29             | Next.js ≥ 16.4.0, `@swc/core` ≥ 1.16.0   |
| `0.3.0` and earlier | 76         | 28             | Next.js ≤ 16.3.8, `@swc/core` 1.15.x     |

A mismatched pair fails the build with:

```
Failed to execute SWC plugin
  1: Failed to deserialize program received from host
```

The `swc_ecma_ast` major changes the CBOR AST wire format, so this is not a per-node degradation you can work around — and a trivial file can still slip through, which means a smoke test on a one-element component does not prove the pair works. Verified for this release: Next.js 16.4.0 and `@swc/core` 1.16.18 accept `0.4.0` and reject `0.3.0`; Next.js 16.3.0, 16.3.8 and `@swc/core` 1.15.47 accept `0.3.0` and reject `0.4.0`.

Check compatibility before pinning a version: [plugins.swc.rs](https://plugins.swc.rs/)

---

## Features

- **Automatic data-testid generation** for functional and class components
- **Component-scoped unique counters** to prevent ID conflicts
- **Predictable naming** like `ComponentName.element`, `ComponentName.element2`
- **Full React support**: Functional components, arrow functions, class components
- **Customizable attributes** (data-testid, data-cy, data-test-id, etc.)
- **Zero configuration** - works out of the box
- **Anonymous default exports** — derives component name from filename
- **JSX member expressions** support (`Modal.Header` → `ComponentName.Header`)
- **Never overrides existing attributes**

## Transformations

### Functional Components

**Before:**

```jsx
function UserCard({ name }) {
  return (
    <div>
      <h3>{name}</h3>
      <button>Follow</button>
      <button>Message</button>
    </div>
  );
}
```

**After:**

```jsx
function UserCard({ name }) {
  return (
    <div data-testid="UserCard.div">
      <h3 data-testid="UserCard.h3">{name}</h3>
      <button data-testid="UserCard.button">Follow</button>
      <button data-testid="UserCard.button2">Message</button>
    </div>
  );
}
```

### Unique Counter System

The plugin uses **component-scoped counters** to ensure uniqueness:

```jsx
function FormComponent() {
  return (
    <div>
      {" "}
      {/* FormComponent.div */}
      <div>First</div> {/* FormComponent.div2 */}
      <div>Second</div> {/* FormComponent.div3 */}
      <button>Save</button> {/* FormComponent.button */}
      <button>Cancel</button> {/* FormComponent.button2 */}
    </div>
  );
}
```

### JSX Member Expressions

```jsx
function ModalComponent() {
  return (
    <Modal.Container>
      {" "}
      {/* ModalComponent.Container */}
      <Modal.Header>Title</Modal.Header> {/* ModalComponent.Header */}
      <Modal.Body>Content</Modal.Body> {/* ModalComponent.Body */}
      <Modal.Header>Second</Modal.Header> {/* ModalComponent.Header2 */}
    </Modal.Container>
  );
}
```

### Class Components

```jsx
class TodoList extends React.Component {
  render() {
    return (
      <div>
        {" "}
        {/* TodoList.div */}
        <h2>My Todos</h2> {/* TodoList.h2 */}
        <ul>
          {" "}
          {/* TodoList.ul */}
          <li>Todo 1</li> {/* TodoList.li */}
          <li>Todo 2</li> {/* TodoList.li2 */}
        </ul>
        <button>Add Todo</button> {/* TodoList.button */}
      </div>
    );
  }
}
```

### Anonymous Default Exports

When there is no explicit component name, the plugin derives it from the filename:

```jsx
// File: UserProfile.tsx
export default () => (
  <div data-testid="UserProfile.div">
    <span data-testid="UserProfile.span">Hello</span>
  </div>
);
```

---

## Configuration Options

Every option defaults to the behaviour that produces the most unique ids. **You should not
need to configure anything** — the flags exist to turn inference *off*, for a codebase
migrating in stages or a callee the heuristics read wrongly. A machine-readable definition
of the whole options object lives in
[`schema/plugin-options.schema.json`](./schema/plugin-options.schema.json).

| Option               | Type       | Default              | Description                                              |
| -------------------- | ---------- | -------------------- | -------------------------------------------------------- |
| `attributes`         | `string[]` | `["data-testid"]`    | Attribute names to add to elements                        |
| `autoIndex`          | `boolean`  | `true`               | Append an index parameter to map callbacks that declare none, so each item gets a unique id |
| `mapMethods`         | `string[]` | `["map", "flatMap"]` | Callee names treated as iteration. Narrow it to leave a name alone, or widen it to add your own iterator |
| `collectionFirstMap` | `boolean`  | `true`               | Recognise `f(collection, callback)` — `_.map`, `Children.map`, `$.map` |
| `nestedIndexes`      | `boolean`  | `true`               | Give an element inside nested callbacks one id segment per level. Turning this off makes nested ids collide across outer iterations |
| `resolveCallbackRefs`| `boolean`  | `true`               | Index a function passed to a map-like call by reference (`items.map(row)`) at its declaration in the same file |
| `indexCounter`       | `boolean`  | `true`               | Supply an index from a call-site counter where the callee passes none — `R.map(fn, items)` |
| `instanceIds`        | `boolean`  | **`false`**          | Prefix a component's ids with the one its parent passed down, so two instances differ. Changes every nested id — see [Limitations](#ids-are-per-component-not-per-instance) |
| `manifestDir`        | `string`   | —                    | ❌ **Not supported** — fails the build, see [Limitations](#manifestdir-is-not-supported) |

If the plugin config cannot be read — most often a value of the wrong type, like
`"autoIndex": "false"` with quotes — **the build fails with an explanatory error**.
Deserialization is all-or-nothing, so one mistyped field would otherwise discard every
other option you set and leave the plugin looking as though it had accepted them.

`instanceIds` is the one option you have to switch **on**: it rewrites ids that already
work, so it cannot be a default. Every other option turns inference *off*.

Setting all six inference flags to their non-default values reproduces the 0.1.x id scheme
exactly:

```json
{
  "autoIndex": false,
  "collectionFirstMap": false,
  "nestedIndexes": false,
  "mapMethods": ["map"],
  "resolveCallbackRefs": false,
  "indexCounter": false
}
```

### Examples

**Multiple testing frameworks:**

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-react-data-testid",
          { "attributes": ["data-testid", "data-cy", "data-test-id"] }
        ]
      ]
    }
  }
}
```

**Cypress only:**

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        ["swc-plugin-react-data-testid", { "attributes": ["data-cy"] }]
      ]
    }
  }
}
```

**Disable plugin:**

```json
{
  "jsc": {
    "experimental": {
      "plugins": [["swc-plugin-react-data-testid", { "attributes": [] }]]
    }
  }
}
```

---

## Testing Integration

### Jest + React Testing Library

```javascript
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import UserCard from "./UserCard";

test("should interact with generated test ids", async () => {
  render(<UserCard name="John" />);

  const followButton = screen.getByTestId("UserCard.button");
  const messageButton = screen.getByTestId("UserCard.button2");

  await userEvent.click(followButton);

  expect(screen.getByTestId("UserCard.div")).toBeInTheDocument();
});
```

### Cypress

```javascript
describe("UserCard Component", () => {
  it("should interact with elements", () => {
    cy.mount(<UserCard name="John" />);

    cy.get('[data-cy="UserCard.button"]').click();
    cy.get('[data-cy="UserCard.button2"]').should("be.visible");
  });
});
```

### Playwright

```javascript
import { test, expect } from "@playwright/test";

test("user card interactions", async ({ page }) => {
  await page.goto("/user-profile");

  await page.locator('[data-testid="UserCard.button"]').click();
  await expect(page.locator('[data-testid="UserCard.div"]')).toBeVisible();
});
```

---

## Supported React Patterns

| Pattern                | Supported | Example                                                |
| ---------------------- | --------- | ------------------------------------------------------ |
| Function Components    | ✅        | `function MyComponent() {}`                            |
| Arrow Functions        | ✅        | `const MyComponent = () => {}`                         |
| Class Components       | ✅        | `class MyComponent extends React.Component {}`         |
| Anonymous Exports      | ✅        | `export default () => {}` (name derived from filename) |
| JSX Member Expressions | ✅        | `<Modal.Header>` → `ComponentName.Header`              |
| Fragments              | ✅        | `<>` and `<React.Fragment>`                            |
| Conditional Rendering  | ✅        | Multiple return statements                             |
| Existing Attributes    | ✅        | Never overrides existing `data-testid`                 |
| Self-Closing Elements  | ✅        | `<img />`, `<input />`                                 |
| Nested Components      | ✅        | Deep nesting with unique counters                      |
| `.map()` with index    | ✅        | `items.map((it, i) => …)` → `{"Comp.li-" + i}`         |
| `.map()` without index | ✅        | An index parameter is appended → `{"Comp.li-" + i}`    |
| `.map()` with a rest param | ✅    | Reads `args[1]` — the signature is left untouched      |
| `.flatMap()`           | ✅        | Detected like `.map()`                                 |
| `_.map(items, fn)`     | ✅        | Collection first, callback second — lodash, `Children.map`, `$.map` |
| `.map(fn, thisArg)`    | ✅        | Still `Array.prototype.map`, so the index is real      |
| RxJS `map(fn)`         | ✅        | The operator passes `(value, index)`                   |
| Nested `.map()`        | ✅        | One index per level → `{"Comp.td-" + r + "-" + c}`     |
| `R.map(fn, items)`     | ✅        | Ramda passes no index, so a call-site counter supplies one |
| `items.map(renderItem)`| ✅        | The referenced function is indexed where it is declared |
| `items.map((it, [i]) => …)` | ✅   | The first identifier the pattern binds is used, guarded  |
| Two instances of a component | ✅  | With [`instanceIds: true`](#ids-are-per-component-not-per-instance) — off by default, it changes existing ids |
| `items.map(imported)`  | ⚠️        | The declaration is in another file, out of the plugin's reach — static id |
| `R.map((it, i) => …)`  | ⚠️        | An index **you** declare is used as written, so a callee that passes none yields `-undefined` |

---

## Limitations

### Iteration forms that do not get unique ids

Elements rendered inside a map callback get a unique runtime id per item —
`data-testid={"List.li-" + index}` → `List.li-0`, `List.li-1`, … The plugin infers the
index from whatever is available, in order of confidence, and **no configuration is
needed for any of this**:

```jsx
// all of these produce List.li-0, List.li-1, List.li-2
{items.map((item, index) => <li>{item}</li>)}   // declared — used as-is
{items.map((item, index = 0) => <li>{item}</li>)}  // a default is unwrapped
{items.map((...args) => <li>{args[0]}</li>)}    // read from args[1], signature untouched
{items.map((item) => <li>{item}</li>)}          // appended → (item, __testidIdx0) => …
{items.flatMap((item) => <li>{item}</li>)}      // flatMap passes an index too
{_.map(items, (item) => <li>{item}</li>)}       // collection first, callback second
{Children.map(children, (child) => <li>{child}</li>)}
{$.map(items, (item) => <li>{item}</li>)}
{items.map((item) => <li>{item}</li>, this)}    // .map(cb, thisArg) is still Array.map
{items$.pipe(map((item) => <li>{item}</li>))}   // RxJS operator: passes (value, index)
```

Nested callbacks contribute one index segment each, so cells stay unique across rows:

```jsx
// Table.td-0-0, Table.td-0-1, Table.td-1-0, …
{rows.map((cells) => <tr>{cells.map((cell) => <td>{cell}</td>)}</tr>)}
```

A callback that is passed **by reference** has no parameter list at the call site, so the
index is added where the function is declared instead. The declaration may sit anywhere in
the same file — before the call, after it, or hoisted:

```jsx
{items.map(renderItem)}                         // renderItem.li-0, renderItem.li-1, …
const renderItem = (item) => <li>{item}</li>;   // ← the index lands here
```

The function is matched by name, so two same-named functions in one file are both indexed
if either is used as a callback — harmless, since the segment is guarded. Appending to a
declaration changes its arity for every caller, not only the map call; `resolveCallbackRefs:
false` turns the whole behaviour off.

A destructuring pattern in the index slot leaves nothing to append, but it does bind
something the source pulled out of that argument, and the first such binding is used:

```jsx
{items.map((item, { index }) => <li/>)}         // List.li-0, List.li-1, …
```

**Where the callee turns out to pass no index at all** — ramda, lodash-fp, or a helper of
your own named `map` — the plugin supplies one itself, from a counter bound just outside
the call:

```jsx
{R.map((item) => <li/>, items)}                 // List.li-0, List.li-1, …
// emitted as ((c) => R.map((item, i = c++) => …, items))(0)
```

The counter is reset every time the call runs, so ids track item positions rather than
climbing across renders. It is only used where the call has a collection argument after
the callback: a curried `map(fn)` evaluates once and the function it returns may be stored
and reused, where a counter would drift. That shape keeps the guarded index, which drops
out to the plain static `List.li` — never `List.li-undefined`:

```jsx
{R.map((item) => <li/>)(items)}                 // → "List.li" on every item
```

These forms still get one static id for every item:

```jsx
// ⚠️ every <li> gets the identical id "List.li"
{items.map(imported)}             // declared in another file, out of reach
{items.map((item, []) => <li/>)}  // the pattern binds nothing at all
```

And one form can still produce `-undefined`, because it is your own assertion rather than
the plugin's guess — an index **you** declare is used exactly as written, on the callee
you wrote it for:

```jsx
{R.map((item, i) => <li/>, items)}              // ⚠️ "List.li-undefined" — ramda passes no i
{R.map((item) => <li/>, items)}                 // ✅ let the plugin infer it instead
```

Duplicate testids are legal HTML but will break `getByTestId`-style queries that expect
one match. To fix a static id, switch to a supported form or set the attribute yourself —
the plugin never overwrites one that is already present.

Note that JSX appearing **outside** the callback — in the callee or a later argument —
correctly keeps a static id, since the index variable does not exist in that scope:

```jsx
// <Header> → "List.Header" (static)   <li> → {"List.li-" + i} (dynamic)
{[<Header key="h" />, ...items].map((item, i) => <li>{item}</li>)}
```

### Ids are per component, not per instance

By default the counter resets for each component, so a component rendered more than once
repeats its ids:

```jsx
<Badge /> <Badge />   // both render <span data-testid="Badge.span">
```

The map index only disambiguates the element *at* the call site, not the elements inside
the component:

```jsx
{items.map((item) => <Badge />)}
// <Badge> itself → "List.Badge-0", "List.Badge-1"  ✅
// the <span> inside Badge → "Badge.span" on both   ⚠️
```

No id assigned at compile time from one source node can tell instances apart — the plugin
cannot know how many times a component will be rendered. What it *can* do is let each
instance carry a prefix down to its children at runtime, which is what `instanceIds` does:

```json
{ "instanceIds": true }
```

```jsx
const Badge = ({ label }) => <span>{label}</span>;
// becomes ({ "data-testid": p, label }) => <span data-testid={(p || "Badge") + ".span"}>

<Toolbar>
  <Badge /> <Badge />
</Toolbar>
// Toolbar.Badge.span  and  Toolbar.Badge2.span    ← distinct

{items.map((item) => <Badge />)}
// Rows.Badge-0.span, Rows.Badge-1.span, …          ← distinct per item
```

The prop is the attribute the plugin already injects on the child element (or the first of
them, if you configured several), so nothing has to be wired up by hand. A component that
receives no prefix — rendered at the top level, or by code the plugin never saw — falls
back to its own name and keeps exactly the id it had before.

**This is off by default because it is a breaking change**: every id below the top level
gains a path prefix, so existing selectors for nested components stop matching. Turn it on
when you are ready to update them. Three things to know before you do:

- Ids become as deep as the render tree. `Page.Card2.Row-3.Badge.span` is precise but long,
  and it changes whenever a component is moved to a different parent.
- A component whose props you destructure gains one property in the pattern. If it also
  has a rest element, that rest no longer collects `data-testid`, so a `{...rest}` spread
  onto the root element no longer overwrites the id the plugin put there.
- Class components read `this.props`. Inside a `function` expression `this` is something
  else, so elements there keep the plain component-name prefix; use an arrow callback if
  you want the prefix to reach them.

### The appended index parameter

When a callback declares no index, the plugin appends one named `__testidIdx<n>`:

```jsx
{items.map((item) => <li>{item}</li>)}
// becomes
{items.map((item, __testidIdx0) => (
  <li data-testid={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>
))}
```

The name is generated per file and checked against every identifier in the callback, so
it cannot shadow anything the body uses. It is only added when a testid actually
references it — a callback that renders no JSX keeps its original signature.

**Why the guard.** Whether the function you called really passes an index is a static
guess — `map(fn)` is RxJS's operator (which does) and curried ramda (which does not),
with identical syntax. The guard is what makes guessing *broadly* safe: if an index
arrives the id is `List.li-0`, identical to writing the index yourself, and if it does not
the segment drops out to exactly the static `List.li`. A wrong guess can never produce
`List.li-undefined`, which is why the plugin infers everywhere it plausibly can rather
than only where it is certain. This matters for a helper of your own named `map`:

```jsx
const map = (list, fn) => list.map((v) => fn(v));   // never passes an index
{map(items, (item) => <li>{item}</li>)}             // → "List.li" for every item
```

The test is `=== void 0` rather than a truthiness check, because index `0` is falsy and
`_.map(someObject, cb)` passes a string key rather than a number. An index **you** write
is never guarded — declaring `(item, index) =>` asserts the binding is real, so the id
stays the compact `{"List.li-" + index}`. A default value is unwrapped, so
`(item, index = 0) =>` works the same way.

Generated names are deterministic for a given file: the counter increments in traversal
order, so the same source always yields the same names. They are not stable *across*
edits — adding a map callback earlier in the file renumbers the ones after it. If you
snapshot-test transpiled output rather than rendered DOM, expect churn there; snapshots
of rendered markup only ever see the resulting `data-testid` values.

One side effect to be aware of: appending a plain parameter changes the callback's
`Function.length`, so a callee that branches on callback arity sees a different value.
Array methods, lodash and `React.Children.map` do not, but if you pass callbacks to
something that does, set `autoIndex: false`. (The counter form appends a parameter *with
a default*, which leaves `Function.length` untouched.)

Set `autoIndex: false` to turn appending off entirely and keep static ids for callbacks
that declare no index.

### Upgrading from 0.1.x

0.2.0 changes ids that were previously duplicated. Selectors that target those elements
need updating:

| Source | 0.1.x | 0.2.0 |
| ------ | ----- | ----- |
| `items.map((item) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `items.flatMap((item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `_.map(items, (item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `rows.map((r, i) => <tr>{r.map((c, j) => <td/>)}</tr>)` | `Table.td-0`, repeated per row | `Table.td-0-0`, `Table.td-0-1`, … |

Ids that were already unique are unchanged.

Each row has its own opt-out, so a large codebase can migrate in stages rather than all at
once:

| Row | Opt-out |
| --- | ------- |
| 1 | `autoIndex: false` |
| 2 | `mapMethods: ["map"]` |
| 3 | `collectionFirstMap: false` |
| 4 | `nestedIndexes: false` |

Set all four and the ids are exactly 0.1.x's. Each one you leave at its default is a set of
elements that get unique ids instead of duplicates, so treat them as temporary. To pin an
individual id permanently, set the attribute explicitly on the element — the plugin never
overwrites one that is already present.

### Upgrading again: the last three duplicated forms

Three shapes that previously gave every item the same id now get unique ones. If you wrote
selectors against those duplicated ids, they change:

| Source | Before | Now | Opt-out |
| ------ | ------ | --- | ------- |
| `R.map((item) => <li/>, items)` | `List.li` on every item | `List.li-0`, `List.li-1`, … | `indexCounter: false` |
| `items.map(renderItem)` | `renderItem.li` on every item | `renderItem.li-0`, … | `resolveCallbackRefs: false` |
| `items.map((item, [i]) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … | — |

Ids that were already unique are unchanged, and `instanceIds` is off unless you turn it
on, so nothing below the top level gains a prefix by upgrading.

### `manifestDir` is not supported

The `manifestDir` option cannot work and will **fail your build with an explanatory
error**. SWC runs Wasm plugins inside a WASI sandbox with no preopened directories, so
the plugin cannot write to any path on disk — this is a property of the SWC plugin
runtime, not a misconfiguration. Remove the option to unblock your build.

Making it work requires a redesign that does not use the filesystem (emitting testids
through SWC's plugin metadata channel, or collecting them in a host-side post-pass).
`scripts/merge-manifest.mjs` is retained for that future design.

---

## Example Applications

Every example carries the same **iteration-forms checklist** — each map-like shape the
plugin recognises, paired with the exact ids it must produce. The browser examples check
themselves and report a banner; the test examples assert the same expectations. Between
them, all four SWC hosts are covered:

| Example | Host | How the checklist appears |
| ------- | ---- | ------------------------- |
| `example/vite` | `@vitejs/plugin-react-swc` | self-checking page, dev + `vite build` |
| `example/nextjs-swc` | Next.js / Turbopack | self-checking page, after hydration of the prerendered, minified output |
| `example/jest` | `@swc/jest` | `src/__tests__/IterationForms.test.jsx` |
| `example/vitest` | Vitest + `@vitejs/plugin-react-swc` | `MapHosts.test.jsx` (real libraries) + the vite page rendered headlessly |

The checklist covers a declared index, an index behind a default value, an appended index,
both rest-parameter shapes, `flatMap`, `.map(callback, thisArg)`, collection-first
`map(items, callback)`, `React.Children.map`, the operator form `map(callback)`, nested
callbacks, an id written by hand, and a callee that passes no index at all — which must
degrade to the static id. The pages add two page-wide invariants: no id reads
`-undefined`, and no inferred id is duplicated.

### Vite — a self-checking page

```bash
cd example/vite
pnpm install
pnpm dev         # then open http://localhost:5173
```

The "Iteration Forms — self-checking" section reports `✅ 15/15 checks passed`, or lists
the expected and actual ids for anything that failed — so verifying a build is reading one
banner rather than inspecting elements by hand.

That page is also rendered headlessly in `example/vitest`
(`src/__tests__/IterationForms.test.jsx`), so the banner turning red fails CI.

### Next.js

```bash
cd example/nextjs-swc
pnpm install
pnpm dev         # then open http://localhost:3000
```

The same self-checking section runs after hydration, so a green banner means the ids
survived a **prerendered, minified Turbopack build** — the only host where a generated
identifier that failed to bind would surface as a `ReferenceError` rather than merely a
wrong id. `pnpm build` prerenders the page to `.next/server/pages/index.html`, where
the ids can be read directly.

### Jest (`@swc/jest`)

```bash
cd example/jest
pnpm install
pnpm test
```

`@testing-library/react` tests asserting the plugin injected the expected attributes.
`src/__tests__/IterationForms.test.jsx` is the fastest way to see exactly what ids each
iteration form produces.

### Vitest

```bash
cd example/vitest
pnpm install
pnpm test
```

Same assertions via `@vitejs/plugin-react-swc`, plus `src/__tests__/MapHosts.test.jsx`,
which exercises the iteration forms against the **real** libraries rather than stand-ins —
`lodash` (arrays *and* objects, where the callback receives a string key) and `jquery`.

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

### Commands

```bash
cargo test                                          # Run fixture tests
UPDATE=1 cargo test                                 # Regenerate fixture outputs
cargo build --target wasm32-wasip1 --release        # Build WASM binary
npm run build                                       # Build WASM and copy to plugin.wasm
```

### Test Structure

Tests use Rust fixture tests in `tests/fixture.rs`. Each fixture is a directory under `tests/fixture*/` with an `input.tsx` and `output.tsx`. Run `UPDATE=1 cargo test` to regenerate outputs after algorithm changes.

### Test Coverage

Current coverage: **96.78% line coverage** across 91 tests (26 unit tests + 65 fixture tests).

To measure coverage locally:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov
```

#### Why 100% is not achievable

The remaining ~5% is the `process_transform` function — the `#[plugin_transform]` WASM entry point that SWC calls at runtime. It requires `TransformPluginProgramMetadata`, a type provided by the SWC WASM runtime that cannot be constructed in a Rust unit test. This function is intentionally thin (it delegates immediately to `parse_options` and `ReactDataTestIdTransform`, both of which are fully tested). Covering it would require a full WASM integration test outside of `cargo test`.

---

## Contributors

Thanks to these wonderful people who have contributed to this project:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<table>
  <tr>
    <td align="center">
      <a href="https://github.com/Kazaz-Or">
        <img src="https://github.com/Kazaz-Or.png?size=40" width="40px;" alt="Or Kazaz"/>
        <br />
        <sub><b>Or Kazaz</b></sub>
      </a>
      <br />
      <a href="#maintenance-Kazaz-Or" title="Maintenance">🚧</a>
      <a href="https://github.com/Kazaz-Or/swc-plugin-react-data-testid/commits?author=Kazaz-Or" title="Code">💻</a>
      <a href="#design-Kazaz-Or" title="Design">🎨</a>
      <a href="https://github.com/Kazaz-Or/swc-plugin-react-data-testid/commits?author=Kazaz-Or" title="Documentation">📖</a>
      <a href="#ideas-Kazaz-Or" title="Ideas, Planning, & Feedback">🤔</a>
    </td>
  </tr>
</table>
<!-- ALL-CONTRIBUTORS-LIST:END -->

### How to Contribute

- **Report bugs** by opening an issue
- **Suggest features** or improvements
- **Improve documentation**
- **Add test cases** (fixture-based)
- **Submit pull requests**

<details>
<summary><strong>Contribution Process</strong></summary>

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add fixture tests
4. Ensure all tests pass (`cargo test`)
5. Commit your changes
6. Push to the branch
7. Open a Pull Request

</details>

---

## License

MIT © [Oakblu](https://github.com/Oakblu)

---

**Made with love for better testing**
