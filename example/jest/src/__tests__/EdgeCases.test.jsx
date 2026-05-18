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

describe("EdgeCases — map callback shares component counter", () => {
  it("all runtime <li> nodes share the same source-level testid", () => {
    const { container } = render(<MapDemo />);

    // Three runtime nodes, but all share the same single source node → same testid
    const items = container.querySelectorAll('[data-testid="MapDemo.li"]');
    expect(items).toHaveLength(3);
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
