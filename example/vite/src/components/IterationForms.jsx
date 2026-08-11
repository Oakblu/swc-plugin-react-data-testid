import React, { useEffect, useRef, useState } from "react";

// Every iteration form the plugin infers an index for, each paired with the exact ids it
// must produce. The expectations are checked against the live DOM by <IterationForms/>
// below, so this page reports PASS/FAIL instead of asking you to read DevTools.

const FRUITS = ["Apple", "Banana", "Cherry"];
const PAIR = ["Apple", "Banana"];

// ─── 1. Declared index ────────────────────────────────────────────────────────
// The parameter is trusted: the id is emitted unguarded, `"Declared.li-" + i`.
function Declared() {
  return <ul>{FRUITS.map((fruit, i) => <li key={i}>{fruit}</li>)}</ul>;
}

// ─── 2. Index behind a default value ──────────────────────────────────────────
// `(item, i = 0)` still binds one identifier, so it counts as declared.
function DefaultValued() {
  return <ul>{FRUITS.map((fruit, i = 0) => <li key={i}>{fruit}</li>)}</ul>;
}

// ─── 3. No index declared ─────────────────────────────────────────────────────
// A parameter is appended. Guarded, because the plugin cannot know the callee passes one.
function Appended() {
  return <ul>{FRUITS.map((fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// ─── 4. Rest parameter only ───────────────────────────────────────────────────
// The index is already collected — the id reads `args[1]` and the signature is untouched.
function RestOnly() {
  return <ul>{PAIR.map((...args) => <li key={args[0]}>{args[0]}</li>)}</ul>;
}

// ─── 5. Trailing rest parameter ───────────────────────────────────────────────
// Same, one slot along: `rest[0]`.
function TrailingRest() {
  return <ul>{PAIR.map((fruit, ...rest) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// ─── 6. flatMap ───────────────────────────────────────────────────────────────
function FlatMapped() {
  const groups = [["Apple", "Banana"], ["Cherry"]];
  return <ul>{groups.flatMap((group) => <li key={group[0]}>{group.join(" + ")}</li>)}</ul>;
}

// ─── 7. .map(callback, thisArg) ───────────────────────────────────────────────
// A second argument does not stop this being Array.prototype.map.
function WithThisArg() {
  return <ul>{PAIR.map((fruit) => <li key={fruit}>{fruit}</li>, undefined)}</ul>;
}

// ─── 8. Collection-first map, index passed ────────────────────────────────────
// The shape lodash / jQuery / React.Children use: `f(collection, callback)`. The real
// libraries are asserted in example/vitest/src/__tests__/MapHosts.test.jsx; this stand-in
// keeps this example dependency-free.
const helpers = {
  map: (list, fn) => list.map((value, i) => fn(value, i)),
};

function CollectionFirst() {
  return <ul>{helpers.map(PAIR, (fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// ─── 9. React.Children.map ────────────────────────────────────────────────────
// The real collection-first callee that ships with React itself.
function ChildrenMapped({ children }) {
  return <ul>{React.Children.map(children, (child) => <li>{child}</li>)}</ul>;
}

// ─── 10. Collection-first map that passes NO index ────────────────────────────
// The inferred segment is guarded, so it drops out entirely — a plain static id, never
// `-undefined`. This is what licenses inferring so broadly: a wrong guess costs nothing.
const map = (list, fn) => list.map((value) => fn(value));

function NoIndexPassed() {
  return <ul>{map(PAIR, (fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// ─── 11. Operator style — map(callback) returning a function ──────────────────
// The RxJS shape. `map` is shadowed locally so the callee is still named `map`.
function OperatorStyle() {
  const map = (fn) => (list) => list.map((value, i) => fn(value, i));
  const pipe = (list, op) => op(list);
  return <ul>{pipe(PAIR, map((fruit) => <li key={fruit}>{fruit}</li>))}</ul>;
}

// ─── 12. Nested callbacks ─────────────────────────────────────────────────────
// One segment per level, otherwise every row would repeat the same cell ids.
function Nested() {
  const rows = [["r1c1", "r1c2"], ["r2c1", "r2c2"]];
  return (
    <table>
      <tbody>
        {rows.map((cells) => (
          <tr key={cells[0]}>
            {cells.map((cell) => (
              <td key={cell}>{cell}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// ─── 13. An id already written by hand ────────────────────────────────────────
// Never overwritten, not even inside a map. This is how you pin an id permanently.
function Pinned() {
  return <ul>{PAIR.map((fruit) => <li key={fruit} data-testid="pinned-by-hand">{fruit}</li>)}</ul>;
}

// ─── The checklist ────────────────────────────────────────────────────────────

const CHECKS = [
  {
    id: "declared",
    title: "Declared index",
    source: "items.map((fruit, i) => <li/>)",
    note: "Trusted, so the id is unguarded.",
    element: "li",
    expected: ["Declared.li-0", "Declared.li-1", "Declared.li-2"],
    render: () => <Declared />,
  },
  {
    id: "default-valued",
    title: "Index behind a default value",
    source: "items.map((fruit, i = 0) => <li/>)",
    note: "0.2.0: previously fell back to one shared static id.",
    element: "li",
    expected: ["DefaultValued.li-0", "DefaultValued.li-1", "DefaultValued.li-2"],
    render: () => <DefaultValued />,
  },
  {
    id: "appended",
    title: "No index declared",
    source: "items.map((fruit) => <li/>)",
    note: "0.2.0: a parameter is appended. Was `Appended.li` on every item.",
    element: "li",
    expected: ["Appended.li-0", "Appended.li-1", "Appended.li-2"],
    render: () => <Appended />,
  },
  {
    id: "rest-only",
    title: "Rest parameter only",
    source: "items.map((...args) => <li/>)",
    note: "Reads args[1]. Nothing appended, so Function.length is unchanged.",
    element: "li",
    expected: ["RestOnly.li-0", "RestOnly.li-1"],
    render: () => <RestOnly />,
  },
  {
    id: "trailing-rest",
    title: "Trailing rest parameter",
    source: "items.map((fruit, ...rest) => <li/>)",
    note: "Reads rest[0].",
    element: "li",
    expected: ["TrailingRest.li-0", "TrailingRest.li-1"],
    render: () => <TrailingRest />,
  },
  {
    id: "flatmap",
    title: "flatMap",
    source: "groups.flatMap((group) => <li/>)",
    note: "0.2.0: flatMap was not recognised as iteration at all.",
    element: "li",
    expected: ["FlatMapped.li-0", "FlatMapped.li-1"],
    render: () => <FlatMapped />,
  },
  {
    id: "this-arg",
    title: "map(callback, thisArg)",
    source: "items.map((fruit) => <li/>, undefined)",
    note: "A second argument does not stop this being Array.prototype.map.",
    element: "li",
    expected: ["WithThisArg.li-0", "WithThisArg.li-1"],
    render: () => <WithThisArg />,
  },
  {
    id: "collection-first",
    title: "Collection-first map",
    source: "helpers.map(items, (fruit) => <li/>)",
    note: "0.2.0: the lodash / jQuery / Children.map shape. Was a shared static id.",
    element: "li",
    expected: ["CollectionFirst.li-0", "CollectionFirst.li-1"],
    render: () => <CollectionFirst />,
  },
  {
    id: "children-map",
    title: "React.Children.map",
    source: "React.Children.map(children, (child) => <li/>)",
    note: "The collection-first callee React itself ships.",
    element: "li",
    expected: ["ChildrenMapped.li-0", "ChildrenMapped.li-1"],
    render: () => (
      <ChildrenMapped>
        <span>Apple</span>
        <span>Banana</span>
      </ChildrenMapped>
    ),
  },
  {
    id: "operator-style",
    title: "Operator style — map(callback)",
    source: "pipe(items, map((fruit) => <li/>))",
    note: "The RxJS shape: the operator passes (value, index).",
    element: "li",
    expected: ["OperatorStyle.li-0", "OperatorStyle.li-1"],
    render: () => <OperatorStyle />,
  },
  {
    id: "nested",
    title: "Nested callbacks",
    source: "rows.map((cells) => <tr>{cells.map((cell) => <td/>)}</tr>)",
    note: "0.2.0: one segment per level. Every row used to repeat the same cell ids.",
    element: "td",
    expected: ["Nested.td-0-0", "Nested.td-0-1", "Nested.td-1-0", "Nested.td-1-1"],
    render: () => <Nested />,
  },
  {
    id: "no-index-passed",
    title: "A map that passes no index",
    source: "map(items, (fruit) => <li/>)   // your own helper",
    note: "The guard drops the segment: a static id, never `-undefined`.",
    element: "li",
    expected: ["NoIndexPassed.li", "NoIndexPassed.li"],
    render: () => <NoIndexPassed />,
  },
  {
    id: "pinned",
    title: "An id written by hand",
    source: '<li data-testid="pinned-by-hand"/>',
    note: "Never overwritten, not even inside a map.",
    element: "li",
    expected: ["pinned-by-hand", "pinned-by-hand"],
    render: () => <Pinned />,
  },
];

const idsUnder = (root, selector) =>
  Array.from(root.querySelectorAll(selector)).map((el) => el.getAttribute("data-testid"));

const same = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);

function runChecks(root) {
  const results = CHECKS.map((check) => {
    const scope = root.querySelector(`[data-check="${check.id}"]`);
    const actual = scope ? idsUnder(scope, check.element) : [];
    return { ...check, actual, pass: same(actual, check.expected) };
  });

  // Two invariants that span more than one demo. `-undefined` is checked across the whole
  // document, since a single unguarded segment anywhere would be a shipped bug.
  const everyId = idsUnder(root, "[data-testid]");
  results.push({
    id: "no-undefined",
    title: "No id in the whole document reads `-undefined`",
    source: "every [data-testid] on the page",
    note: "The guard exists to make this true even when a callee passes no index.",
    expected: ["none"],
    actual: idsUnder(document.body, "[data-testid]").filter((v) => v.includes("undefined")),
    get pass() {
      return this.actual.length === 0;
    },
  });

  const dynamic = everyId.filter((v) => v && /-\d+(-\d+)*$/.test(v));
  const duplicates = dynamic.filter((v, i) => dynamic.indexOf(v) !== i);
  results.push({
    id: "no-duplicates",
    title: "Every inferred id on this page is unique",
    source: "every id ending in an index segment",
    note: "A duplicate here means an index was missed.",
    expected: ["none"],
    actual: Array.from(new Set(duplicates)),
    get pass() {
      return this.actual.length === 0;
    },
  });

  return results;
}

export default function IterationForms() {
  const rootRef = useRef(null);
  const [results, setResults] = useState(null);

  useEffect(() => {
    if (rootRef.current) setResults(runChecks(rootRef.current));
  }, []);

  const failures = results ? results.filter((r) => !r.pass) : [];
  const status = !results
    ? { text: "running…", tone: "#64748b" }
    : failures.length === 0
      ? { text: `✅ ${results.length}/${results.length} checks passed`, tone: "#16a34a" }
      : { text: `❌ ${failures.length} of ${results.length} checks failed`, tone: "#dc2626" };

  return (
    <div ref={rootRef}>
      <div
        style={{
          padding: "0.75rem 1rem",
          marginBottom: "1rem",
          borderRadius: 8,
          border: `2px solid ${status.tone}`,
          color: status.tone,
          fontWeight: 700,
        }}
      >
        {status.text}
        {failures.length > 0 && (
          <ul style={{ margin: "0.5rem 0 0", fontWeight: 400 }}>
            {failures.map((f) => (
              <li key={f.id}>
                <code>{f.title}</code> — expected{" "}
                <code>{JSON.stringify(f.expected)}</code>, got{" "}
                <code>{JSON.stringify(f.actual)}</code>
              </li>
            ))}
          </ul>
        )}
      </div>

      {CHECKS.map((check) => {
        const result = results && results.find((r) => r.id === check.id);
        return (
          <section
            key={check.id}
            data-check={check.id}
            style={{
              padding: "0.75rem 1rem",
              marginBottom: "0.75rem",
              borderRadius: 8,
              borderLeft: `4px solid ${result ? (result.pass ? "#16a34a" : "#dc2626") : "#cbd5e1"}`,
              background: "rgba(148, 163, 184, 0.08)",
            }}
          >
            <strong>
              {result ? (result.pass ? "✅" : "❌") : "⏳"} {check.title}
            </strong>
            <pre style={{ margin: "0.35rem 0", overflowX: "auto" }}>
              <code>{check.source}</code>
            </pre>
            <small style={{ display: "block", opacity: 0.75 }}>{check.note}</small>
            <small style={{ display: "block", marginTop: "0.35rem" }}>
              expected <code>{check.expected.join(", ")}</code>
            </small>
            {check.render()}
          </section>
        );
      })}
    </div>
  );
}
