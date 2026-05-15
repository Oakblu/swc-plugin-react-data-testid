# swc-plugin-react-data-testid

[![npm version](https://img.shields.io/npm/v/swc-plugin-react-data-testid.svg)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Automatically adds `data-testid` attributes to React JSX elements at build time. Uses **component-scoped unique counters** for predictable, collision-free test IDs.

> For Babel projects, use [babel-plugin-react-data-testid-generator](https://www.npmjs.com/package/babel-plugin-react-data-testid-generator) instead.

## Installation

```bash
npm install --save-dev swc-plugin-react-data-testid
# or
yarn add -D swc-plugin-react-data-testid
```

## Setup

### Next.js

```js
// next.config.js
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    swcPlugins: [
      ['swc-plugin-react-data-testid', {}]
    ]
  }
}

module.exports = nextConfig
```

### Vite (`@vitejs/plugin-react-swc`)

```js
// vite.config.js
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

export default defineConfig({
  plugins: [
    react({
      plugins: [['swc-plugin-react-data-testid', {}]]
    })
  ]
})
```

### `@swc/jest`

```js
// jest.config.js
module.exports = {
  transform: {
    '^.+\\.[jt]sx?$': ['@swc/jest', {
      jsc: {
        experimental: {
          plugins: [['swc-plugin-react-data-testid', {}]]
        }
      }
    }]
  }
}
```

### `.swcrc`

```json
{
  "jsc": {
    "experimental": {
      "plugins": [["swc-plugin-react-data-testid", {}]]
    }
  }
}
```

## How It Works

Given this component:

```jsx
function UserCard({ name }) {
  return (
    <div>
      <h3>{name}</h3>
      <button>Follow</button>
      <button>Message</button>
    </div>
  )
}
```

The plugin outputs:

```jsx
function UserCard({ name }) {
  return (
    <div data-testid="UserCard.div">
      <h3 data-testid="UserCard.h3">{name}</h3>
      <button data-testid="UserCard.button">Follow</button>
      <button data-testid="UserCard.button2">Message</button>
    </div>
  )
}
```

Counters are scoped per component, so `button` in `UserCard` and `button` in `LoginForm` are independent. The plugin never overwrites an attribute that already exists.

### JSX member expressions

```jsx
function ModalComponent() {
  return (
    <Modal.Container>
      <Modal.Header>Title</Modal.Header>  {/* ModalComponent.Header */}
      <Modal.Body>Content</Modal.Body>    {/* ModalComponent.Body */}
      <Modal.Header>Second</Modal.Header> {/* ModalComponent.Header2 */}
    </Modal.Container>
  )
}
```

### Class components

```jsx
class TodoList extends React.Component {
  render() {
    return (
      <div>          {/* TodoList.div */}
        <ul>         {/* TodoList.ul */}
          <li>A</li> {/* TodoList.li */}
          <li>B</li> {/* TodoList.li2 */}
        </ul>
      </div>
    )
  }
}
```

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `attributes` | `string[]` | `["data-testid"]` | Attribute names to inject |

### Multiple attributes

Inject multiple attributes at once (e.g. to support both RTL and Cypress):

```json
["swc-plugin-react-data-testid", { "attributes": ["data-testid", "data-cy"] }]
```

### Playwright (`data-test-id`)

```json
["swc-plugin-react-data-testid", { "attributes": ["data-test-id"] }]
```

### Disable the plugin

Pass an empty array to skip all injection:

```json
["swc-plugin-react-data-testid", { "attributes": [] }]
```

## Production builds

Consider disabling in production to reduce HTML payload:

```js
// next.config.js
const isDev = process.env.NODE_ENV !== 'production'

const nextConfig = {
  experimental: {
    swcPlugins: isDev
      ? [['swc-plugin-react-data-testid', {}]]
      : []
  }
}

module.exports = nextConfig
```

## Supported patterns

| Pattern | Supported |
|---------|-----------|
| Function components | ✅ |
| Arrow function components | ✅ |
| Class components (`render()`) | ✅ |
| JSX member expressions | ✅ |
| Fragments (`<>`, `<React.Fragment>`) | ✅ |
| Conditional / multiple returns | ✅ |
| Self-closing elements | ✅ |
| Existing attribute — never overridden | ✅ |
| Anonymous default exports | ⚠️ skipped (no component name) |

## SWC version compatibility

SWC plugins are compiled against a specific `swc_core` version and **must match** the version your consumer uses. Using a mismatched plugin will produce a runtime error or silently do nothing.

| Plugin version | `swc_core` | Next.js | `@swc/core` |
|---------------|------------|---------|-------------|
| `0.1.x` | `65.x` | `16.2.x` | `1.10.18` |

**How to find your `@swc/core` version:**

```bash
npm ls @swc/core
# or, inside a Next.js project:
npm ls next | grep swc
```

If your SWC version isn't listed above, check [plugins.swc.rs](https://plugins.swc.rs/) to find which plugin version to use, or open an issue.

## License

MIT © [Or Kazaz](https://github.com/Kazaz-Or)
