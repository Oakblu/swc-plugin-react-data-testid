import pluginTester from "babel-plugin-tester";
import plugin from "../index";

pluginTester({
  plugin,
  pluginName: "babel-plugin-react-data-testid-generator",
  babelOptions: { parserOpts: { plugins: ["jsx"] } },
  tests: [
    {
      title: "should add data-testid to functional component elements",
      code: `
        function MyComponent() {
          return <div><button>Click me</button></div>;
        }
      `,
      output: `
        function MyComponent() {
          return (
            <div data-testid="MyComponent.div">
              <button data-testid="MyComponent.button">Click me</button>
            </div>
          );
        }
      `,
    },
    {
      title: "should add data-testid to arrow function component elements",
      code: `
        const MyComponent = () => {
          return <div><span>Hello</span></div>;
        };
      `,
      output: `
        const MyComponent = () => {
          return (
            <div data-testid="MyComponent.div">
              <span data-testid="MyComponent.span">Hello</span>
            </div>
          );
        };
      `,
    },
    {
      title: "should add data-testid to class component render method",
      code: `
        class MyComponent extends React.Component {
          render() {
            return <div><p>Content</p></div>;
          }
        }
      `,
      output: `
        class MyComponent extends React.Component {
          render() {
            return (
              <div data-testid="MyComponent.div">
                <p data-testid="MyComponent.p">Content</p>
              </div>
            );
          }
        }
      `,
    },
    {
      title: "should not override existing data-testid",
      code: `
        function MyComponent() {
          return <div data-testid="existing"><button>Click</button></div>;
        }
      `,
      output: `
        function MyComponent() {
          return (
            <div data-testid="existing">
              <button data-testid="MyComponent.button">Click</button>
            </div>
          );
        }
      `,
    },
    {
      title: "should work with custom attributes",
      code: `
        function MyComponent() {
          return <div><button>Click</button></div>;
        }
      `,
      output: `
        function MyComponent() {
          return (
            <div data-cy="MyComponent.div">
              <button data-cy="MyComponent.button">Click</button>
            </div>
          );
        }
      `,
      pluginOptions: { attributes: ["data-cy"] },
    },
    {
      title: "should handle JSX member expressions",
      code: `
        function MyComponent() {
          return <Modal.Header><Modal.Title>Title</Modal.Title></Modal.Header>;
        }
      `,
      output: `
        function MyComponent() {
          return (
            <Modal.Header data-testid="MyComponent.Header">
              <Modal.Title data-testid="MyComponent.Title">Title</Modal.Title>
            </Modal.Header>
          );
        }
      `,
    },
    {
      title: "should skip components without identifiable names and no filename",
      code: `
        export default () => {
          return <div><button>Click</button></div>;
        };
      `,
      output: `
        export default () => {
          return (
            <div>
              <button>Click</button>
            </div>
          );
        };
      `,
    },
    {
      title: "should use filename for anonymous default export arrow function",
      babelOptions: { filename: "/src/pages/about.tsx", parserOpts: { plugins: ["jsx"] } },
      code: `
        export default () => {
          return <div><button>Click</button></div>;
        };
      `,
      output: `
        export default () => {
          return (
            <div data-testid="About.div">
              <button data-testid="About.button">Click</button>
            </div>
          );
        };
      `,
    },
    {
      title: "should use filename for anonymous default export function declaration",
      babelOptions: { filename: "/app/pages/contact.jsx", parserOpts: { plugins: ["jsx"] } },
      code: `
        export default function() {
          return <div><h1>Contact</h1><form><input /></form></div>;
        }
      `,
      output: `
        export default function () {
          return (
            <div data-testid="Contact.div">
              <h1 data-testid="Contact.h1">Contact</h1>
              <form data-testid="Contact.form">
                <input data-testid="Contact.input" />
              </form>
            </div>
          );
        }
      `,
    },
    {
      title: "should prefer explicit function name over filename",
      babelOptions: { filename: "/src/pages/about.tsx", parserOpts: { plugins: ["jsx"] } },
      code: `
        export default function AboutPage() {
          return <div><h1>About</h1></div>;
        }
      `,
      output: `
        export default function AboutPage() {
          return (
            <div data-testid="AboutPage.div">
              <h1 data-testid="AboutPage.h1">About</h1>
            </div>
          );
        }
      `,
    },
    {
      title: "should handle multiple attributes",
      code: `
        function MyComponent() {
          return <div><button>Click</button></div>;
        }
      `,
      output: `
        function MyComponent() {
          return (
            <div data-cy="MyComponent.div" data-testid="MyComponent.div">
              <button data-cy="MyComponent.button" data-testid="MyComponent.button">
                Click
              </button>
            </div>
          );
        }
      `,
      pluginOptions: { attributes: ["data-testid", "data-cy"] },
    },
    {
      title: "should handle duplicate elements with unique counters",
      code: `
        function FormComponent() {
          return (
            <div>
              <div>First div</div>
              <div>Second div</div>
              <button>First button</button>
              <button>Second button</button>
            </div>
          );
        }
      `,
      output: `
        function FormComponent() {
          return (
            <div data-testid="FormComponent.div">
              <div data-testid="FormComponent.div2">First div</div>
              <div data-testid="FormComponent.div3">Second div</div>
              <button data-testid="FormComponent.button">First button</button>
              <button data-testid="FormComponent.button2">Second button</button>
            </div>
          );
        }
      `,
    },
    {
      title: "should handle conditional rendering with unique IDs",
      code: `
        function ConditionalComponent({ showFirst }) {
          if (showFirst) {
            return <div>First</div>;
          }
          return <div>Second</div>;
        }
      `,
      output: `
        function ConditionalComponent({ showFirst }) {
          if (showFirst) {
            return <div data-testid="ConditionalComponent.div">First</div>;
          }
          return <div data-testid="ConditionalComponent.div2">Second</div>;
        }
      `,
    },
    {
      title: "should reset counters per component",
      code: `
        function FirstComponent() {
          return <div><span>First</span></div>;
        }
        
        function SecondComponent() {
          return <div><span>Second</span></div>;
        }
      `,
      output: `
        function FirstComponent() {
          return (
            <div data-testid="FirstComponent.div">
              <span data-testid="FirstComponent.span">First</span>
            </div>
          );
        }
        function SecondComponent() {
          return (
            <div data-testid="SecondComponent.div">
              <span data-testid="SecondComponent.span">Second</span>
            </div>
          );
        }
      `,
    },
  ],
});
