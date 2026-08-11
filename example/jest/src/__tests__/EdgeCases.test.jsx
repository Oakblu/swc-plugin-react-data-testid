import "@testing-library/jest-dom";
import React, { useState } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// ── Ternary rendering ──────────────────────────────────────────────────────────
function TernaryDemo() {
  const [show, setShow] = useState(true);
  return (
    <div>
      {show ? <p>True branch</p> : <em>False branch</em>}
      <button onClick={() => setShow((s) => !s)}>Toggle</button>
    </div>
  );
}

// ── Counter isolation across sibling components ────────────────────────────────
function IsoAlpha() {
  return (
    <div>
      <button>Alpha</button>
    </div>
  );
}

function IsoBeta() {
  return (
    <div>
      <button>Beta</button>
    </div>
  );
}

// ── Many identical elements ────────────────────────────────────────────────────
function ManyItemsList() {
  return (
    <ul>
      <li>One</li>
      <li>Two</li>
      <li>Three</li>
      <li>Four</li>
      <li>Five</li>
    </ul>
  );
}

// ── Map callback — single source node ─────────────────────────────────────────
function MapDemo() {
  const fruits = ["Apple", "Banana", "Cherry"];
  return (
    <ul>
      {fruits.map((fruit, i) => (
        <li key={i}>{fruit}</li>
      ))}
    </ul>
  );
}

// ── Map callbacks with no index parameter ──────────────────────────────────────
// The source never declares an index; the plugin appends one so each item still
// gets a unique id.
function NoIndexDemo() {
  const fruits = ["Apple", "Banana", "Cherry"];
  return (
    <ul>
      {fruits.map((fruit) => (
        <li>{fruit}</li>
      ))}
    </ul>
  );
}

function FlatMapDemo() {
  const groups = [["a", "b"], ["c"]];
  return (
    <ul>
      {groups.flatMap((group) => (
        <li>{group.join(", ")}</li>
      ))}
    </ul>
  );
}

// Collection first, callback second — the shape lodash's _.map(collection, cb) uses.
function ChildrenMapDemo({ children }) {
  return <div>{React.Children.map(children, (child) => <span>{child}</span>)}</div>;
}

function NestedMapDemo() {
  const rows = [
    ["r1c1", "r1c2"],
    ["r2c1", "r2c2"],
  ];
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

// A helper of your own named `map` matches the collection-first shape, so the plugin
// appends an index — but this one never passes it. The guarded segment must drop out,
// leaving the plain static id rather than "CustomMapDemo.li-undefined".
const map = (list, fn) => list.map((value) => fn(value));

function CustomMapDemo() {
  const items = ["x", "y"];
  return <ul>{map(items, (item) => <li>{item}</li>)}</ul>;
}

// ── Nested component scope ─────────────────────────────────────────────────────
function NestedDemo() {
  const Inner = () => <span>inner badge</span>;
  return (
    <div>
      <Inner />
      <p>outer paragraph</p>
    </div>
  );
}

// ── Existing attribute is never overridden ─────────────────────────────────────
function PreservedId() {
  return (
    <div>
      <button data-testid="my-custom-id">Custom</button>
      <button>Generated</button>
    </div>
  );
}

describe("EdgeCases — ternary rendering", () => {
  it("tags both branches; only the active one is in the DOM", async () => {
    const user = userEvent.setup();
    render(<TernaryDemo />);

    expect(screen.getByTestId("TernaryDemo.div")).toBeInTheDocument();
    expect(screen.getByTestId("TernaryDemo.p")).toHaveTextContent("True branch");
    expect(screen.queryByTestId("TernaryDemo.em")).not.toBeInTheDocument();

    await user.click(screen.getByTestId("TernaryDemo.button"));

    expect(screen.queryByTestId("TernaryDemo.p")).not.toBeInTheDocument();
    expect(screen.getByTestId("TernaryDemo.em")).toHaveTextContent("False branch");
  });
});

describe("EdgeCases — counter isolation", () => {
  it("each component resets its counter independently", () => {
    const { container } = render(
      <div>
        <IsoAlpha />
        <IsoBeta />
      </div>
    );

    // IsoAlpha: div + button (not div2/button2)
    expect(container.querySelector('[data-testid="IsoAlpha.div"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="IsoAlpha.button"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="IsoAlpha.div2"]')).not.toBeInTheDocument();

    // IsoBeta: its own fresh counter — div + button, not div2
    expect(container.querySelector('[data-testid="IsoBeta.div"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="IsoBeta.button"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="IsoBeta.div2"]')).not.toBeInTheDocument();
  });
});

describe("EdgeCases — many identical elements", () => {
  it("appends numeric suffix from the second occurrence onward", () => {
    const { container } = render(<ManyItemsList />);

    expect(container.querySelector('[data-testid="ManyItemsList.ul"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ManyItemsList.li"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ManyItemsList.li2"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ManyItemsList.li3"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ManyItemsList.li4"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ManyItemsList.li5"]')).toBeInTheDocument();
  });
});

describe("EdgeCases — map callback with index param gets dynamic testids", () => {
  it("each runtime <li> node gets a unique index-suffixed testid", () => {
    const { container } = render(<MapDemo />);

    // The .map() callback declares an index param, so the plugin emits a dynamic
    // expression — data-testid={"MapDemo.li-" + i} — giving each item a unique id.
    expect(container.querySelector('[data-testid="MapDemo.li-0"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="MapDemo.li-1"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="MapDemo.li-2"]')).toBeInTheDocument();

    const items = container.querySelectorAll('[data-testid^="MapDemo.li-"]');
    expect(items).toHaveLength(3);
  });
});

describe("EdgeCases — map callbacks without an index parameter", () => {
  it("appends an index so every item gets a unique id", () => {
    const { container } = render(<NoIndexDemo />);

    expect(container.querySelector('[data-testid="NoIndexDemo.li-0"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="NoIndexDemo.li-1"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="NoIndexDemo.li-2"]')).toBeInTheDocument();
    expect(container.querySelectorAll('[data-testid^="NoIndexDemo.li-"]')).toHaveLength(3);
  });

  it("detects flatMap", () => {
    const { container } = render(<FlatMapDemo />);

    expect(container.querySelector('[data-testid="FlatMapDemo.li-0"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="FlatMapDemo.li-1"]')).toBeInTheDocument();
  });

  it("detects a callback passed as the second argument", () => {
    const { container } = render(
      <ChildrenMapDemo>
        <b>one</b>
        <b>two</b>
      </ChildrenMapDemo>
    );

    expect(container.querySelector('[data-testid="ChildrenMapDemo.span-0"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="ChildrenMapDemo.span-1"]')).toBeInTheDocument();
  });

  it("keeps ids unique across nested callbacks", () => {
    const { container } = render(<NestedMapDemo />);

    // One index segment per level — an inner index alone would repeat per row.
    for (const id of ["td-0-0", "td-0-1", "td-1-0", "td-1-1"]) {
      expect(
        container.querySelector(`[data-testid="NestedMapDemo.${id}"]`)
      ).toBeInTheDocument();
    }
    expect(container.querySelectorAll('[data-testid^="NestedMapDemo.td-"]')).toHaveLength(4);
  });

  it("degrades to the static id when the callee does not pass an index", () => {
    const { container } = render(<CustomMapDemo />);

    const items = container.querySelectorAll("li");
    expect(items).toHaveLength(2);
    // The appended parameter is never filled, so the guard drops the segment entirely.
    for (const li of items) {
      expect(li.getAttribute("data-testid")).toBe("CustomMapDemo.li");
    }
    expect(container.innerHTML).not.toContain("undefined");
  });
});

describe("EdgeCases — nested component scope", () => {
  it("Inner and NestedDemo have separate scopes", () => {
    const { container } = render(<NestedDemo />);

    // Inner (var-declarator arrow) opens its own scope
    expect(container.querySelector('[data-testid="Inner.span"]')).toBeInTheDocument();

    // NestedDemo owns the outer elements
    expect(container.querySelector('[data-testid="NestedDemo.div"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="NestedDemo.p"]')).toBeInTheDocument();
  });
});

describe("EdgeCases — existing attributes are preserved", () => {
  it("never overwrites a manually set data-testid", () => {
    const { container } = render(<PreservedId />);

    const customBtn = container.querySelector('[data-testid="my-custom-id"]');
    expect(customBtn).toBeInTheDocument();
    expect(customBtn).toHaveTextContent("Custom");

    // The second button still gets a generated id — counter increments even for skipped elements
    expect(container.querySelector('[data-testid="PreservedId.button2"]')).toBeInTheDocument();
  });
});
