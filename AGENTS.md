# swc-plugin-react-data-testid — AI Agent Reference

This file describes the plugin's behavior precisely so AI agents can predict generated
`data-testid` values, write accurate tests, and avoid stale selectors.

## What the plugin does

At build time, this SWC plugin walks every React component's JSX tree and injects one or
more test attributes (default: `data-testid`) onto every JSX element that does not already
carry that attribute. The injected value encodes the component name and element type in a
way that is **deterministic given only the source file**.

## Naming algorithm — step by step

```
testid = "{ComponentName}.{elementId}"
```

where `elementId` is derived by a **per-component counter** keyed on element type:

| Occurrence within the component | elementId    |
|---------------------------------|--------------|
| 1st `<div>`                     | `div`        |
| 2nd `<div>`                     | `div2`       |
| 3rd `<div>`                     | `div3`       |
| 1st `<button>`                  | `button`     |
| 2nd `<button>`                  | `button2`    |

Counters are **reset for every component** — two different components both have their own
`div` counter starting at 1.

**JSX member expressions** use only the right-hand property name:

```
<Modal.Header> → elementId = "Header"
<Modal.Body>   → elementId = "Body"
```

So `<Modal.Header>` inside `Foo` → `Foo.Header`.

**Namespaced elements** (`<svg:image>`) are skipped entirely (no attribute injected, no
counter increment).

**Counting order** follows the depth-first AST traversal order — the same order elements
appear when you read the source top-to-bottom, outside-in. A parent element is counted
before its children.

## Component detection

The plugin recognises these patterns as component entry points and scopes counters to each:

| Source pattern                                  | ComponentName used          |
|-------------------------------------------------|-----------------------------|
| `function MyComp() {}`                          | `MyComp`                    |
| `const MyComp = () => {}`                       | `MyComp`                    |
| `const MyComp = function() {}`                  | `MyComp`                    |
| `class MyComp extends React.Component { render() {} }` | `MyComp`           |
| `export default function Named() {}`            | `Named`                     |
| `export default function() {}` (anonymous)      | derived from filename (see below) |
| `export default () => {}` (anonymous arrow)     | derived from filename (see below) |

**Anonymous default export filename fallback**: when there is no explicit name, the plugin
uppercases the first character of the file's stem (no extension, no dots in the stem).

```
src/userCard.tsx  → UserCard
src/MyModal.tsx   → MyModal
src/index.tsx     → (skipped — "index" with no dots is fine, but "index.test.tsx" is rejected)
```

Stems that contain a dot (`index.test`, `foo.spec`) or are empty (`.hidden`) return no
component name → JSX inside such anonymous exports receives no attribute.

## Skip conditions

The plugin skips injection (and does NOT increment the counter) when:

- The JSX element already has the target attribute (`data-testid` or whatever is configured).
- The element is a namespaced JSX element (`svg:image`, `xml:lang`, etc.).
- The JSX is not inside a recognised component scope.

## Configuration

Plugin options (passed as second element of the plugin tuple):

```json
{ "attributes": ["data-testid"] }
```

| Field        | Type       | Default              | Meaning                                    |
|--------------|------------|----------------------|--------------------------------------------|
| `attributes` | `string[]` | `["data-testid"]`    | Attributes to inject; empty array disables |
| `manifestDir`| `string`   | `null`               | Directory for per-file manifest JSON files |

When `attributes` contains multiple names, **all** are injected on every element (unless
that specific attribute already exists on the element).

## Predicting testids — worked example

```tsx
// src/LoginForm.tsx
function LoginForm() {
  return (
    <form>            {/* LoginForm.form   (1st form)   */}
      <input />       {/* LoginForm.input  (1st input)  */}
      <input />       {/* LoginForm.input2 (2nd input)  */}
      <button>Go</button> {/* LoginForm.button (1st button) */}
    </form>
  );
}
```

To predict testids for any component:

1. Identify the component name (function/class/variable name, or filename stem for anonymous defaults).
2. Walk JSX elements in source order (depth-first, parent before children).
3. Maintain a counter map per component, keyed on element name (or member-expression property).
4. For each element: increment its counter, output `"{ComponentName}.{elementName}"` (no
   suffix for count=1, append the count as a decimal for count≥2).
5. Skip elements that already carry the configured attribute.

## Manifest file format

When `manifestDir` is set, each transformed source file writes one JSON file inside that
directory:

```json
{
  "file": "src/LoginForm.tsx",
  "testids": [
    "LoginForm.form",
    "LoginForm.input",
    "LoginForm.input2",
    "LoginForm.button"
  ]
}
```

Merge all per-file JSONs into a single manifest with:

```bash
node scripts/merge-manifest.mjs .testid-manifest testid-manifest.json
```

The merged format is:

```json
{
  "src/LoginForm.tsx": ["LoginForm.form", "LoginForm.input", "LoginForm.input2", "LoginForm.button"],
  "src/UserCard.tsx": ["UserCard.div", "UserCard.h3", "UserCard.button", "UserCard.button2"]
}
```

## Writing tests against generated testids

Use the attribute value directly in your test framework selectors:

```ts
// React Testing Library
screen.getByTestId("LoginForm.button")

// Playwright
page.locator('[data-testid="LoginForm.button"]')

// Cypress
cy.get('[data-testid="LoginForm.button"]')
```

When an element already has `data-testid` set manually in source, the plugin leaves it
untouched — use that manual value in tests, not the generated one.
