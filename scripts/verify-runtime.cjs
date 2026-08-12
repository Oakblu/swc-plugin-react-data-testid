#!/usr/bin/env node
// Runs the plugin through a real SWC host and then EXECUTES the result — once as
// emitted, once minified with mangling.
//
// Why this exists, on top of the fixtures:
//   * fixtures never cross the plugin FFI boundary, so they cannot prove plugin.wasm
//     itself works (rule 7);
//   * fixtures never minify, so they cannot catch a generated identifier reference that
//     does not bind — that fails only in a production build (rules 8 and 10);
//   * fixtures never run, so they cannot tell a unique id from a plausible-looking one.
//     Every check below asserts the ids of *rendered* markup.
//
// Usage:  node scripts/verify-runtime.cjs
// It borrows @swc/core and react-dom from example/jest, so that example must be
// installed, and loads ../plugin.wasm by absolute path — so run `npm run build` first,
// or it will silently verify the previous binary.
const path = require("path");
const assert = require("assert");
const { createRequire } = require("module");

const EXAMPLE = path.join(__dirname, "..", "example", "jest");
const PLUGIN_WASM = path.join(__dirname, "..", "plugin.wasm");
let req;
try {
  req = createRequire(path.join(EXAMPLE, "package.json"));
  req("@swc/core");
} catch (error) {
  console.error(
    `cannot load @swc/core from ${EXAMPLE} — install that example first ` +
      `(cd example/jest && pnpm install).\n${error.message}`,
  );
  process.exit(2);
}
const swc = req("@swc/core");
const { renderToStaticMarkup } = req("react-dom/server");

const SOURCE = `
// ramda calls its callback with the element only — no index. Faithful stand-in: what
// matters is the shape of the call and the arity of the invocation.
const R = { map: (fn, list) => list.map((x) => fn(x)) };

// The callee passes no index at all. Before the call-site counter, every <li> here
// carried the identical id "Ramda.li".
export const Ramda = ({ items }) => <ul>{R.map((item) => <li>{item}</li>, items)}</ul>;

// The callback is a bare reference: there is no parameter list at the call site, so the
// index is added where the function is declared — below the call that reaches it.
export const ByRef = ({ items }) => <ul>{items.map(renderItem)}</ul>;
const renderItem = (item) => <li>{item}</li>;

// A destructuring pattern in the index slot. Array.prototype.map passes a number there,
// which cannot be destructured — so this shape only occurs with a callee that passes
// something else, and these are those callees.
const withMeta = { map: (list, fn) => list.map((x, i) => fn(x, { index: i })) };
export const DestructuredObject = ({ items }) => (
  <ul>{withMeta.map(items, (item, { index }) => <li>{item}</li>)}</ul>
);
const tupled = { map: (fn, list) => list.map((x, i) => fn(x, [i])) };
export const DestructuredArray = ({ items }) => (
  <ul>{tupled.map((item, [i]) => <li>{item}</li>, items)}</ul>
);

// Two instances of one component (instanceIds).
const Badge = ({ label }) => <span>{label}</span>;
export const Toolbar = () => (
  <div>
    <Badge label="a" />
    <Badge label="b" />
  </div>
);

// Regression guards for the paths that already worked.
export const Plain = ({ items }) => <ul>{items.map((item) => <li>{item}</li>)}</ul>;
export const RamdaDeclared = ({ items }) => (
  <ul>{R.map((item, i) => <li>{item}</li>, items)}</ul>
);
export const Nested = ({ rows }) => (
  <table>{rows.map((cells) => <tr>{cells.map((cell) => <td>{cell}</td>)}</tr>)}</table>
);
`;

function compile(options) {
  return swc.transformSync(SOURCE, {
    filename: "verify.jsx",
    module: { type: "commonjs" },
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      transform: { react: { runtime: "automatic" } },
      target: "es2022",
      // The binary at the repo root, by absolute path: this checks what `npm run build`
      // produced, not whatever copy an example happens to be holding (rule 9).
      experimental: { plugins: [[PLUGIN_WASM, options]] },
    },
  }).code;
}

function load(code) {
  const module = { exports: {} };
  new Function("require", "module", "exports", code)(req, module, module.exports);
  return module.exports;
}

// Every data-testid in the rendered markup, in document order.
function ids(element) {
  return [...renderToStaticMarkup(element).matchAll(/data-testid="([^"]*)"/g)].map((m) => m[1]);
}

let failures = 0;
function check(label, actual, expected) {
  try {
    assert.deepStrictEqual(actual, expected);
    console.log(`  ok    ${label}`);
  } catch {
    failures++;
    console.log(`  FAIL  ${label}`);
    console.log(`        expected ${JSON.stringify(expected)}`);
    console.log(`        actual   ${JSON.stringify(actual)}`);
  }
}

function run(label, code) {
  console.log(`\n${label}`);
  const m = load(code);
  const items = ["a", "b", "c"];

  check("R.map(fn, items) — callee passes no index", ids(m.Ramda({ items })), [
    "Ramda.ul", "Ramda.li-0", "Ramda.li-1", "Ramda.li-2",
  ]);
  check("items.map(renderItem) — callback by reference", ids(m.ByRef({ items })), [
    "ByRef.ul", "renderItem.li-0", "renderItem.li-1", "renderItem.li-2",
  ]);
  check("map(items, (it, { index }) => …) — destructured slot", ids(m.DestructuredObject({ items })), [
    "DestructuredObject.ul", "DestructuredObject.li-0", "DestructuredObject.li-1", "DestructuredObject.li-2",
  ]);
  // The occupied slot must win over the counter — nothing may be appended here.
  check("map((it, [i]) => …, items) — destructured slot beats the counter", ids(m.DestructuredArray({ items })), [
    "DestructuredArray.ul", "DestructuredArray.li-0", "DestructuredArray.li-1", "DestructuredArray.li-2",
  ]);
  check("items.map((item) => …)", ids(m.Plain({ items })), [
    "Plain.ul", "Plain.li-0", "Plain.li-1", "Plain.li-2",
  ]);
  check("nested callbacks — one segment per level", ids(m.Nested({ rows: [["a", "b"], ["c"]] })), [
    "Nested.table", "Nested.tr-0", "Nested.td-0-0", "Nested.td-0-1", "Nested.tr-1", "Nested.td-1-0",
  ]);
  // An index the source DECLARES is used exactly as written, even where the callee
  // passes none: the plugin does not put a default on a parameter its author owns, and
  // the declaration is the author's own assertion that the binding is real. This is the
  // one shape that still yields "-undefined", and it is documented as such.
  check("R.map((item, i) => …) — declared index, callee passes none", ids(m.RamdaDeclared({ items })), [
    "RamdaDeclared.ul", "RamdaDeclared.li-undefined", "RamdaDeclared.li-undefined", "RamdaDeclared.li-undefined",
  ]);
}

function runInstance(label, code) {
  console.log(`\n${label}`);
  const m = load(code);
  check("two instances of <Badge/> no longer repeat ids", ids(m.Toolbar()), [
    "Toolbar.div", "Toolbar.Badge.span", "Toolbar.Badge2.span",
  ]);
}

const minify = (code) => swc.minifySync(code, { compress: true, mangle: true, module: true }).code;

const plain = compile({});
const instance = compile({ instanceIds: true });

run("default options — as emitted", plain);
run("default options — MINIFIED (mangle: true)", minify(plain));
runInstance("instanceIds — as emitted", instance);
runInstance("instanceIds — MINIFIED (mangle: true)", minify(instance));

if (failures > 0) {
  console.error(`\n${failures} runtime check(s) failed`);
  process.exit(1);
}
console.log("\nall runtime checks passed");
