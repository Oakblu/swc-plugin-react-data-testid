# react-data-testid plugins (swc-plugin-react-data-testid)

[![npm version (babel)](https://img.shields.io/npm/v/babel-plugin-react-data-testid-generator.svg?label=babel)](https://www.npmjs.com/package/babel-plugin-react-data-testid-generator)
[![npm version (swc)](https://img.shields.io/npm/v/swc-plugin-react-data-testid.svg?label=swc)](https://www.npmjs.com/package/swc-plugin-react-data-testid)
[![npm downloads](https://img.shields.io/npm/dm/babel-plugin-react-data-testid-generator.svg)](https://www.npmjs.com/package/babel-plugin-react-data-testid-generator)
[![Build Status](https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator/workflows/🔍%20PR%20Validations/badge.svg)](https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator/actions)
[![Coverage](https://img.shields.io/badge/coverage-96%25-brightgreen.svg)](https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> 🧪 **Automatic React data-testid generation with unique IDs — available for both Babel and SWC**

Automatically adds `data-testid` attributes to your React components during the build process, making it easier to write reliable end-to-end tests. Features **component-scoped unique counters** for predictable, collision-free test IDs.

**Choose your compiler:** [Babel Plugin](#-babel-plugin) · [SWC Plugin](#-swc-plugin)

---

## ⚡ SWC Plugin

> Use this if your project uses SWC (Next.js 12+, `@vitejs/plugin-react-swc`, `@swc/jest`).

### Installation

```bash
npm install --save-dev swc-plugin-react-data-testid
# or
yarn add -D swc-plugin-react-data-testid
```

### Basic Setup

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

### Framework-Specific Setup

<details>
<summary><strong>🚀 Next.js (SWC)</strong></summary>

Configure in `next.config.js`:

```js
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

</details>

<details>
<summary><strong>⚡ Vite with @vitejs/plugin-react-swc</strong></summary>

Configure in `vite.config.js`:

```js
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

</details>

<details>
<summary><strong>🧪 @swc/jest</strong></summary>

Configure in `jest.config.js`:

```js
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

</details>

### With Custom Attributes

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        ["swc-plugin-react-data-testid", { "attributes": ["data-testid", "data-cy"] }]
      ]
    }
  }
}
```

### ⚠️ SWC Version Compatibility

SWC plugins are compiled against a specific `swc_core` version and **must match** what your consumer uses. This plugin targets `swc_core 65.x`. Each SWC minor release may require a new plugin version.

Check compatibility before pinning a version: [plugins.swc.rs](https://plugins.swc.rs/)

---

## 🔵 Babel Plugin

> Use this if your project uses Babel (CRA, older Next.js, custom Babel config).

### Installation

```bash
npm install --save-dev babel-plugin-react-data-testid-generator
# or
yarn add --dev babel-plugin-react-data-testid-generator
```

## ✨ Features

- 🎯 **Automatic data-testid generation** for functional and class components
- 🔢 **Component-scoped unique counters** to prevent ID conflicts
- 📊 **Predictable naming** like `ComponentName.element`, `ComponentName.element2`
- 🎭 **Full React support**: Functional components, arrow functions, class components
- 🔧 **Customizable attributes** (data-testid, data-cy, data-test-id, etc.)
- 🚀 **Zero configuration** - works out of the box
- 📦 **TypeScript support** with full type definitions
- 🌐 **Framework agnostic** - works with Next.js, Vite, CRA, and more
- 🔄 **JSX member expressions** support (`Modal.Header` → `ComponentName.Header`)
- 🛡️ **Never overrides existing attributes**

## 📖 Usage

### Basic Setup

Add the plugin to your `.babelrc.json` or `babel.config.js`:

```json
{
  "plugins": ["babel-plugin-react-data-testid-generator"]
}
```

### Framework-Specific Setup

<details>
<summary><strong>🚀 Next.js</strong></summary>

Create `.babelrc.json` in your project root:

```json
{
  "presets": ["next/babel"],
  "plugins": [
    [
      "babel-plugin-react-data-testid-generator",
      {
        "attributes": ["data-testid"]
      }
    ]
  ]
}
```

</details>

<details>
<summary><strong>⚡ Vite</strong></summary>

Configure in `vite.config.js`:

```javascript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [
          [
            "babel-plugin-react-data-testid-generator",
            {
              attributes: ["data-testid"],
            },
          ],
        ],
      },
    }),
  ],
});
```

</details>

<details>
<summary><strong>📦 Create React App</strong></summary>

**Note**: CRA v5+ ignores custom babel configs. Use Next.js or Vite instead, or eject from CRA.

For ejected CRA, add to `babel.config.js`:

```javascript
module.exports = {
  presets: ["react-app"],
  plugins: [
    [
      "babel-plugin-react-data-testid-generator",
      {
        attributes: ["data-testid"],
      },
    ],
  ],
};
```

</details>

### With Custom Attributes

```json
{
  "plugins": [
    [
      "babel-plugin-react-data-testid-generator",
      {
        "attributes": ["data-testid", "data-cy"]
      }
    ]
  ]
}
```

## 🔄 Transformations

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

### Conditional Rendering

```jsx
function ConditionalComponent({ isLoggedIn }) {
  if (isLoggedIn) {
    return <div>Welcome</div>; // ConditionalComponent.div
  }
  return <div>Please login</div>; // ConditionalComponent.div2
}
```

## ⚙️ Configuration Options

| Option       | Type       | Default           | Description                                 |
| ------------ | ---------- | ----------------- | ------------------------------------------- |
| `attributes` | `string[]` | `["data-testid"]` | Array of attribute names to add to elements |

### Examples

**Multiple testing frameworks:**

```json
{
  "plugins": [
    [
      "babel-plugin-react-data-testid-generator",
      {
        "attributes": ["data-testid", "data-cy", "data-test-id"]
      }
    ]
  ]
}
```

**Cypress only:**

```json
{
  "plugins": [
    [
      "babel-plugin-react-data-testid-generator",
      {
        "attributes": ["data-cy"]
      }
    ]
  ]
}
```

**Disable plugin:**

```json
{
  "plugins": [
    [
      "babel-plugin-react-data-testid-generator",
      {
        "attributes": []
      }
    ]
  ]
}
```

## 🧪 Testing Integration

### Jest + React Testing Library

```javascript
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import UserCard from "./UserCard";

test("should interact with generated test ids", async () => {
  render(<UserCard name="John" />);

  // Predictable test IDs
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

    // Use generated data-cy attributes
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

  // Reliable selectors with generated IDs
  await page.locator('[data-testid="UserCard.button"]').click();
  await expect(page.locator('[data-testid="UserCard.div"]')).toBeVisible();
});
```

## 📁 Example Applications

We provide two complete example applications demonstrating the plugin:

### 🚀 Next.js Example

```bash
cd example/babelrc
npm install
npm run dev
```

**Features:**

- Next.js 13+ with App Router
- Beautiful modern UI with animations
- Interactive components demonstrating test ID generation
- Open [http://localhost:3000](http://localhost:3000)

### ⚡ Vite Example

```bash
cd example/vite
npm install
npm run dev
```

**Features:**

- Vite with React 18
- Lightning-fast HMR
- Same UI as Next.js example
- Open [http://localhost:3001](http://localhost:3001)

**Both examples include:**

- ✅ Interactive forms and buttons
- ✅ Modal components
- ✅ Card layouts
- ✅ Class and functional components
- ✅ Conditional rendering
- ✅ JSX member expressions

**Inspect the DOM** to see the automatically generated `data-testid` attributes!

## 🛠️ Supported React Patterns

| Pattern                | Supported | Example                                        |
| ---------------------- | --------- | ---------------------------------------------- |
| Function Components    | ✅        | `function MyComponent() {}`                    |
| Arrow Functions        | ✅        | `const MyComponent = () => {}`                 |
| Class Components       | ✅        | `class MyComponent extends React.Component {}` |
| Anonymous Exports      | ⚠️        | `export default () => {}` (skipped)            |
| JSX Member Expressions | ✅        | `<Modal.Header>` → `ComponentName.Header`      |
| Fragments              | ✅        | `<>` and `<React.Fragment>`                    |
| Conditional Rendering  | ✅        | Multiple return statements                     |
| Existing Attributes    | ✅        | Never overrides existing `data-testid`         |
| Self-Closing Elements  | ✅        | `<img />`, `<input />`                         |
| Nested Components      | ✅        | Deep nesting with unique counters              |

## 🎯 Best Practices

1. **🏷️ Use PascalCase** for component names to get predictable test IDs
2. **🔍 Inspect Generated IDs** in development to understand the structure
3. **🌍 Environment-Specific** - consider disabling in production:

```javascript
const isProd = process.env.NODE_ENV === "production";

module.exports = {
  plugins: [...(!isProd ? [["babel-plugin-react-data-testid-generator"]] : [])],
};
```

4. **🧪 Test ID Patterns** - use consistent patterns in tests:
   - `ComponentName.elementType` for first occurrence
   - `ComponentName.elementType2` for second occurrence
   - `ComponentName.MemberExpression` for JSX member expressions

## 🔧 Development

### Setup

```bash
git clone https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator.git
cd babel-plugin-react-data-testid-generator
yarn install
```

### Commands

```bash
yarn test              # Run tests
yarn test:watch        # Run tests in watch mode
yarn test:coverage     # Run tests with coverage
yarn build            # Build the plugin
yarn lint             # Lint code
yarn fmt              # Format code
```

### Testing

The plugin has comprehensive test coverage (96%+) including:

- ✅ 62 test cases covering all React patterns
- ✅ Snapshot testing for consistent output
- ✅ Edge case handling
- ✅ CI/CD with coverage thresholds

## 🤝 Contributors

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
      <a href="https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator/commits?author=Kazaz-Or" title="Code">💻</a>
      <a href="#design-Kazaz-Or" title="Design">🎨</a>
      <a href="https://github.com/Kazaz-Or/babel-plugin-react-data-testid-generator/commits?author=Kazaz-Or" title="Documentation">📖</a>
      <a href="#ideas-Kazaz-Or" title="Ideas, Planning, & Feedback">🤔</a>
    </td>
  </tr>
</table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

### How to Contribute

We welcome contributions! Here are some ways you can help:

- 🐛 **Report bugs** by opening an issue
- 💡 **Suggest features** or improvements
- 📖 **Improve documentation**
- 🧪 **Add test cases**
- 🔧 **Submit pull requests**

<details>
<summary><strong>📋 Contribution Process</strong></summary>

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add tests
4. Ensure all tests pass (`yarn test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

</details>

<details>
<summary><strong>🤝 Adding Yourself as a Contributor</strong></summary>

If you contribute to this project, you can add yourself to the contributors list:

1. Install the all-contributors CLI: `npm install -g all-contributors-cli`
2. Add yourself: `all-contributors add <your-username> <contribution-type>`
3. Update the README: `all-contributors generate`

</details>

<details>
<summary><strong>🏷️ Contribution Types</strong></summary>

- 💻 `code` - Code contributions
- 📖 `doc` - Documentation
- 🐛 `bug` - Bug reports
- 💡 `ideas` - Ideas and suggestions
- 🧪 `test` - Tests
- 🎨 `design` - Design
- 🚧 `maintenance` - Maintenance
- 📦 `platform` - Packaging/platform support

</details>

## 📄 License

MIT © [Or Kazaz](https://github.com/Kazaz-Or)

---

## 🙏 Acknowledgments

- Inspired by [babel-plugin-react-data-testid](https://github.com/akameco/babel-plugin-react-data-testid)

**Made with 💙 for better testing**
