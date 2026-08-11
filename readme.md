# swc-plugin-react-data-testid

[![npm version](https://img.shields.io/npm/v/swc-plugin-react-data-testid.svg)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![npm downloads](https://img.shields.io/npm/dm/swc-plugin-react-data-testid.svg)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![Build Status](https://github.com/Oakblu/plugin-react-data-testid-generator/workflows/🦀%20SWC%20Plugin%20CI/badge.svg)](https://github.com/Oakblu/plugin-react-data-testid-generator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Automatically adds `data-testid` attributes to your React components during the build process, making it easier to write reliable end-to-end tests. Features **component-scoped unique counters** for predictable, collision-free test IDs.

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

SWC plugins are compiled against a specific `swc_core` version, which must be compatible with the SWC your bundler uses. This plugin targets `swc_core 76.x`, verified against `@swc/core 1.15.47`.

Starting with `@swc/core 1.15.0`, WASM plugins built with `swc_core >= 47` are ABI-compatible across `@swc/core` versions — so this build works with current Next.js, Vite (`@vitejs/plugin-react-swc`), and `@swc/jest`. That compatibility is partial rather than absolute, so a future SWC release may still require a new plugin version.

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

| Option        | Type       | Default           | Description                                              |
| ------------- | ---------- | ----------------- | -------------------------------------------------------- |
| `attributes`  | `string[]` | `["data-testid"]` | Array of attribute names to add to elements               |
| `autoIndex`   | `boolean`  | `true`            | Append an index parameter to map callbacks that declare none, so each item gets a unique id |
| `manifestDir` | `string`   | —                 | ❌ **Not supported** — fails the build, see [Limitations](#manifestdir-is-not-supported) |

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
| `.flatMap()`           | ✅        | Detected like `.map()`                                 |
| `_.map(items, fn)`     | ✅        | Collection first, callback second — lodash, `Children.map`, `$.map` |
| Nested `.map()`        | ✅        | One index per level → `{"Comp.td-" + r + "-" + c}`     |
| `items.map(renderItem)`| ⚠️        | No function literal to add an index to — static id     |
| `R.map(fn, items)`     | ⚠️        | Callback-first helpers get no index — see [Limitations](#iteration-forms-that-do-not-get-unique-ids) |

---

## Limitations

### Iteration forms that do not get unique ids

Elements rendered inside a map callback get a unique runtime id per item —
`data-testid={"List.li-" + index}` → `List.li-0`, `List.li-1`, … This works whether or
not the source declares an index parameter: if it does not, the plugin appends one.

```jsx
// all of these produce List.li-0, List.li-1, List.li-2
{items.map((item, index) => <li key={index}>{item}</li>)}
{items.map((item) => <li>{item}</li>)}          // → (item, __testidIdx0) => …
{items.flatMap((item) => <li>{item}</li>)}      // flatMap passes an index too
{_.map(items, (item) => <li>{item}</li>)}       // collection first, callback second
{Children.map(children, (child) => <li>{child}</li>)}
```

Nested callbacks contribute one index segment each, so cells stay unique across rows:

```jsx
// Table.td-0-0, Table.td-0-1, Table.td-1-0, …
{rows.map((cells) => <tr>{cells.map((cell) => <td>{cell}</td>)}</tr>)}
```

The remaining forms fall back to a **single static id shared by all items**, because the
plugin assigns ids at compile time from one source node:

```jsx
// ⚠️ every <li> gets the identical id "List.li"
{items.map(renderItem)}                         // callback is not a function literal
{items.map((...args) => <li/>)}                 // nothing may follow a rest parameter
{R.map((item) => <li/>, items)}                 // ramda: callback first, no index
{items.pipe(map((item) => <li/>))}              // bare map(fn): curried / an operator
{items.map((item) => <li/>, this)}              // a second argument makes the shape ambiguous
```

The last three are excluded deliberately: the plugin cannot tell that those callbacks
are called with an index, and an appended parameter that the callee never fills would be
`undefined` in every id. A `.map(cb, thisArg)` callback that *declares* an index still
gets a unique id — the option only governs appending one.

Duplicate testids are legal HTML but will break `getByTestId`-style queries that expect
one match. To fix a static id, switch to a supported form or set the attribute yourself.

Note that JSX appearing **outside** the callback — in the callee or a later argument —
correctly keeps a static id, since the index variable does not exist in that scope:

```jsx
// <Header> → "List.Header" (static)   <li> → {"List.li-" + i} (dynamic)
{[<Header key="h" />, ...items].map((item, i) => <li>{item}</li>)}
```

### The appended index parameter

When a callback declares no index, the plugin appends one named `__testidIdx<n>`:

```jsx
{items.map((item) => <li>{item}</li>)}
// becomes
{items.map((item, __testidIdx0) => <li data-testid={"List.li-" + __testidIdx0}>{item}</li>)}
```

The name is generated per file and checked against every identifier in the callback, so
it cannot shadow anything the body uses. It is only added when a testid actually
references it — a callback that renders no JSX keeps its original signature. Set
`autoIndex: false` to turn this off and keep static ids for callbacks with no index
parameter (useful if you call a custom `map` whose callback receives no index).

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

### Next.js

A complete Next.js example is available in `example/nextjs-swc/`:

```bash
cd example/nextjs-swc
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) and inspect the DOM to see the automatically generated `data-testid` attributes.

### Jest (`@swc/jest`)

A Jest example is available in `example/jest/`. It runs `@testing-library/react` tests that assert the plugin actually injected the expected `data-testid` attributes:

```bash
cd example/jest
npm install
npm test
```

### Vitest

A Vitest example is available in `example/vitest/`. Same assertions as the Jest example, using `@vitejs/plugin-react-swc` to load the plugin:

```bash
cd example/vitest
npm install
npm test
```

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
