import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import React from "react";
import _ from "lodash";
import $ from "jquery";

// Every iteration form the plugin infers an index for, exercised against the REAL
// libraries rather than a stand-in. Transformation happens through
// @vitejs/plugin-react-swc, so these assert the shipped plugin.wasm end to end.

function NoIndexDemo() {
  const items = ["a", "b", "c"];
  return <ul>{items.map((item) => <li>{item}</li>)}</ul>;
}

function FlatMapDemo() {
  const groups = [["a", "b"], ["c"]];
  return <ul>{groups.flatMap((group) => <li>{group.join()}</li>)}</ul>;
}

// lodash: collection first, callback second.
function LodashDemo() {
  const items = ["a", "b"];
  return <ul>{_.map(items, (item) => <li>{item}</li>)}</ul>;
}

// lodash over an object hands the callback a string KEY, not a number — which is why
// the guard tests `=== void 0` rather than truthiness or a numeric comparison.
function LodashObjectDemo() {
  const items = { first: "a", second: "b" };
  return <ul>{_.map(items, (item) => <li>{item}</li>)}</ul>;
}

// jQuery.map(array, callback) also passes (value, index).
function JqueryDemo() {
  const items = ["a", "b"];
  return <ul>{$.map(items, (item) => <li>{item}</li>)}</ul>;
}

// A rest parameter already holds the index; the signature must stay untouched.
function RestDemo() {
  const items = ["a", "b"];
  return <ul>{items.map((...args) => <li>{args[0]}</li>)}</ul>;
}

function NestedDemo() {
  const rows = [["r1c1", "r1c2"], ["r2c1", "r2c2"]];
  return (
    <table>
      <tbody>
        {rows.map((cells) => (
          <tr>
            {cells.map((cell) => (
              <td>{cell}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// A helper of your own named `map` that never passes an index. The guard must drop the
// segment, leaving the plain static id.
const map = (list, fn) => list.map((value) => fn(value));

function CustomMapDemo() {
  const items = ["a", "b"];
  return <ul>{map(items, (item) => <li>{item}</li>)}</ul>;
}

const idsOf = (container, selector) =>
  Array.from(container.querySelectorAll(selector)).map((el) =>
    el.getAttribute("data-testid")
  );

describe("index inference across iteration forms", () => {
  it("appends an index when the callback declares none", () => {
    const { container } = render(<NoIndexDemo />);
    expect(idsOf(container, "li")).toEqual([
      "NoIndexDemo.li-0",
      "NoIndexDemo.li-1",
      "NoIndexDemo.li-2",
    ]);
  });

  it("detects flatMap", () => {
    const { container } = render(<FlatMapDemo />);
    expect(idsOf(container, "li")).toEqual(["FlatMapDemo.li-0", "FlatMapDemo.li-1"]);
  });

  it("detects lodash's collection-first map", () => {
    const { container } = render(<LodashDemo />);
    expect(idsOf(container, "li")).toEqual(["LodashDemo.li-0", "LodashDemo.li-1"]);
  });

  it("uses lodash's string keys when mapping an object", () => {
    const { container } = render(<LodashObjectDemo />);
    expect(idsOf(container, "li")).toEqual([
      "LodashObjectDemo.li-first",
      "LodashObjectDemo.li-second",
    ]);
  });

  it("detects jQuery.map", () => {
    const { container } = render(<JqueryDemo />);
    expect(idsOf(container, "li")).toEqual(["JqueryDemo.li-0", "JqueryDemo.li-1"]);
  });

  it("reads the index out of a rest parameter", () => {
    const { container } = render(<RestDemo />);
    expect(idsOf(container, "li")).toEqual(["RestDemo.li-0", "RestDemo.li-1"]);
  });

  it("keeps ids unique across nested callbacks", () => {
    const { container } = render(<NestedDemo />);
    expect(idsOf(container, "td")).toEqual([
      "NestedDemo.td-0-0",
      "NestedDemo.td-0-1",
      "NestedDemo.td-1-0",
      "NestedDemo.td-1-1",
    ]);
  });

  it("degrades to the static id when the callee passes no index", () => {
    const { container } = render(<CustomMapDemo />);
    expect(idsOf(container, "li")).toEqual(["CustomMapDemo.li", "CustomMapDemo.li"]);
    expect(container.innerHTML).not.toContain("undefined");
  });
});
