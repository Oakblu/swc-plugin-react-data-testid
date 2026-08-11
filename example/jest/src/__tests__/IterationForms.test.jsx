import "@testing-library/jest-dom";
import React from "react";
import { render } from "@testing-library/react";

// Every iteration form the plugin infers an index for, asserted through @swc/jest — the
// same checklist the vite and Next.js example pages verify in a browser, here as a test.
// If you are evaluating the plugin, this file is the fastest way to see exactly what ids
// each shape produces.

const FRUITS = ["Apple", "Banana", "Cherry"];
const PAIR = ["Apple", "Banana"];

// 1. Declared index — trusted, so the id is emitted unguarded.
function Declared() {
  return <ul>{FRUITS.map((fruit, i) => <li key={i}>{fruit}</li>)}</ul>;
}

// 2. Index behind a default value — still binds one identifier, so it counts as declared.
function DefaultValued() {
  return <ul>{FRUITS.map((fruit, i = 0) => <li key={i}>{fruit}</li>)}</ul>;
}

// 3. No index declared — a parameter is appended, guarded.
function Appended() {
  return <ul>{FRUITS.map((fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// 4. Rest parameter only — reads args[1], signature untouched.
function RestOnly() {
  return <ul>{PAIR.map((...args) => <li key={args[0]}>{args[0]}</li>)}</ul>;
}

// 5. Trailing rest parameter — reads rest[0].
function TrailingRest() {
  return <ul>{PAIR.map((fruit, ...rest) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// 6. flatMap.
function FlatMapped() {
  const groups = [["Apple", "Banana"], ["Cherry"]];
  return <ul>{groups.flatMap((group) => <li key={group[0]}>{group.join(" + ")}</li>)}</ul>;
}

// 7. A second argument does not stop this being Array.prototype.map.
function WithThisArg() {
  return <ul>{PAIR.map((fruit) => <li key={fruit}>{fruit}</li>, undefined)}</ul>;
}

// 8. Collection first, callback second — the lodash / jQuery / Children.map shape.
//    example/vitest asserts the same against the real libraries.
const helpers = {
  map: (list, fn) => list.map((value, i) => fn(value, i)),
};

function CollectionFirst() {
  return <ul>{helpers.map(PAIR, (fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// 9. React.Children.map, the real collection-first callee bundled with React.
function ChildrenMapped({ children }) {
  return <ul>{React.Children.map(children, (child) => <li>{child}</li>)}</ul>;
}

// 10. Operator style — map(callback) returning a function. The RxJS shape.
function OperatorStyle() {
  const map = (fn) => (list) => list.map((value, i) => fn(value, i));
  const pipe = (list, op) => op(list);
  return <ul>{pipe(PAIR, map((fruit) => <li key={fruit}>{fruit}</li>))}</ul>;
}

// 11. Nested callbacks — one segment per level.
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

// 12. A helper of your own that passes no index. The guard drops the segment.
const map = (list, fn) => list.map((value) => fn(value));

function NoIndexPassed() {
  return <ul>{map(PAIR, (fruit) => <li key={fruit}>{fruit}</li>)}</ul>;
}

// 13. An id written by hand is never overwritten, not even inside a map.
function Pinned() {
  return <ul>{PAIR.map((fruit) => <li key={fruit} data-testid="pinned-by-hand">{fruit}</li>)}</ul>;
}

const idsOf = (container, selector) =>
  Array.from(container.querySelectorAll(selector)).map((el) =>
    el.getAttribute("data-testid")
  );

describe("iteration forms — every shape that gets a unique id", () => {
  it.each([
    ["a declared index", <Declared />, "li", ["Declared.li-0", "Declared.li-1", "Declared.li-2"]],
    [
      "an index behind a default value",
      <DefaultValued />,
      "li",
      ["DefaultValued.li-0", "DefaultValued.li-1", "DefaultValued.li-2"],
    ],
    [
      "an appended index when none is declared",
      <Appended />,
      "li",
      ["Appended.li-0", "Appended.li-1", "Appended.li-2"],
    ],
    ["a rest parameter (args[1])", <RestOnly />, "li", ["RestOnly.li-0", "RestOnly.li-1"]],
    [
      "a trailing rest parameter (rest[0])",
      <TrailingRest />,
      "li",
      ["TrailingRest.li-0", "TrailingRest.li-1"],
    ],
    ["flatMap", <FlatMapped />, "li", ["FlatMapped.li-0", "FlatMapped.li-1"]],
    [
      "map(callback, thisArg)",
      <WithThisArg />,
      "li",
      ["WithThisArg.li-0", "WithThisArg.li-1"],
    ],
    [
      "a collection-first map",
      <CollectionFirst />,
      "li",
      ["CollectionFirst.li-0", "CollectionFirst.li-1"],
    ],
    [
      "React.Children.map",
      <ChildrenMapped><span>a</span><span>b</span></ChildrenMapped>,
      "li",
      ["ChildrenMapped.li-0", "ChildrenMapped.li-1"],
    ],
    [
      "operator style — map(callback)",
      <OperatorStyle />,
      "li",
      ["OperatorStyle.li-0", "OperatorStyle.li-1"],
    ],
    [
      "nested callbacks",
      <Nested />,
      "td",
      ["Nested.td-0-0", "Nested.td-0-1", "Nested.td-1-0", "Nested.td-1-1"],
    ],
  ])("gives unique ids for %s", (_label, element, selector, expected) => {
    const { container } = render(element);
    expect(idsOf(container, selector)).toEqual(expected);
  });

  it("degrades to the static id when the callee passes no index", () => {
    const { container } = render(<NoIndexPassed />);
    expect(idsOf(container, "li")).toEqual(["NoIndexPassed.li", "NoIndexPassed.li"]);
    expect(container.innerHTML).not.toContain("undefined");
  });

  it("never overwrites an id written by hand", () => {
    const { container } = render(<Pinned />);
    expect(idsOf(container, "li")).toEqual(["pinned-by-hand", "pinned-by-hand"]);
  });

  it("emits no id containing `undefined` for any form", () => {
    const { container } = render(
      <div>
        <Declared />
        <DefaultValued />
        <Appended />
        <RestOnly />
        <TrailingRest />
        <FlatMapped />
        <WithThisArg />
        <CollectionFirst />
        <OperatorStyle />
        <Nested />
        <NoIndexPassed />
      </div>
    );
    const bad = idsOf(container, "[data-testid]").filter((id) => id.includes("undefined"));
    expect(bad).toEqual([]);
  });
});
