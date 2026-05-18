import React, { useState, useMemo, forwardRef } from "react";

// ─── forwardRef ───────────────────────────────────────────────────────────────
// init is Expr::Call — plugin never opens a component scope inside.
// Inner <div>, <label>, <input> get NO data-testid.
const FancyInput = forwardRef((props, ref) => (
  <div>
    <label>Label</label>
    <input ref={ref} {...props} />
  </div>
));

// ─── React.memo ───────────────────────────────────────────────────────────────
// Same reason — the inner <span> gets NO data-testid.
const MemoChip = React.memo(({ text }) => (
  <span>{text}</span>
));

// ─── Nested component ─────────────────────────────────────────────────────────
// Inner is a var-declarator arrow inside Outer → its own scope.
// Expected: Inner.span  /  NestedDemo.div + NestedDemo.p
function NestedDemo() {
  const Inner = () => <span>inner badge</span>;
  return (
    <div>
      <Inner />
      <p>outer paragraph</p>
    </div>
  );
}

// ─── Switch returns ───────────────────────────────────────────────────────────
// Three source-level <div> returns → SwitchDemo.div / .div2 / .div3
function SwitchDemo({ status }) {
  switch (status) {
    case "ok":   return <div>✅ OK</div>;
    case "warn": return <div>⚠️ Warning</div>;
    default:     return <div>❓ Unknown</div>;
  }
}

// ─── useMemo counter sharing ──────────────────────────────────────────────────
// useMemo's arrow is NOT a var-declarator init → no new scope.
// The <span> inside useMemo runs first → HooksDemo.span
// The "direct" <span> in return then becomes HooksDemo.span2
function HooksDemo() {
  const memoSpan = useMemo(() => <span>memoized span (HooksDemo.span)</span>, []);
  return (
    <div>
      {memoSpan}
      <span>direct span (HooksDemo.span2)</span>
    </div>
  );
}

// ─── Ternary rendering ────────────────────────────────────────────────────────
// Expected: TernaryDemo.div / TernaryDemo.p (true) or TernaryDemo.em (false) / TernaryDemo.button
function TernaryDemo() {
  const [show, setShow] = useState(true);
  return (
    <div>
      {show ? <p>Ternary true branch</p> : <em>Ternary false branch</em>}
      <button onClick={() => setShow((s) => !s)}>
        Toggle (now: {show ? "true" : "false"})
      </button>
    </div>
  );
}

// ─── Short-circuit rendering ──────────────────────────────────────────────────
// Expected: ShortCircuitDemo.div / ShortCircuitDemo.p (when true) / ShortCircuitDemo.button
function ShortCircuitDemo() {
  const [show, setShow] = useState(true);
  return (
    <div>
      {show && <p>Short-circuit content</p>}
      <button onClick={() => setShow((s) => !s)}>
        Toggle (now: {show ? "true" : "false"})
      </button>
    </div>
  );
}

// ─── Map callback ─────────────────────────────────────────────────────────────
// Arrow inside .map() is a call argument — current_component stays MapDemo.
// All three <li> nodes in source are ONE source node → MapDemo.ul + MapDemo.li
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

// ─── Counter isolation ────────────────────────────────────────────────────────
// Expected: IsoAlpha.div + IsoAlpha.button, then IsoBeta.div + IsoBeta.button
// (NOT IsoAlpha.div2 or IsoAlpha.button2)
function IsoAlpha() {
  return <div><button>Alpha</button></div>;
}
function IsoBeta() {
  return <div><button>Beta</button></div>;
}

// ─── Main showcase ────────────────────────────────────────────────────────────
export default function EdgeCasesShowcase() {
  return (
    <div>
      <section>
        <h3>Ternary rendering</h3>
        <small>Expected: <code>TernaryDemo.p</code> or <code>TernaryDemo.em</code> depending on state</small>
        <TernaryDemo />
      </section>

      <section>
        <h3>Short-circuit rendering</h3>
        <small>Expected: <code>ShortCircuitDemo.p</code> present/absent depending on state</small>
        <ShortCircuitDemo />
      </section>

      <section>
        <h3>Map callback</h3>
        <small>All three runtime &lt;li&gt; nodes share <code>MapDemo.li</code> — one source node</small>
        <MapDemo />
      </section>

      <section>
        <h3>useMemo counter sharing</h3>
        <small>
          Inner arrow is not a var-declarator init — useMemo JSX runs first, consuming{" "}
          <code>HooksDemo.span</code>; direct span becomes <code>HooksDemo.span2</code>
        </small>
        <HooksDemo />
      </section>

      <section>
        <h3>Nested component scope</h3>
        <small>
          <code>Inner.span</code> inside Inner; <code>NestedDemo.div</code> +{" "}
          <code>NestedDemo.p</code> inside Outer
        </small>
        <NestedDemo />
      </section>

      <section>
        <h3>forwardRef — inner elements have NO testid</h3>
        <small>
          <code>EdgeCasesShowcase.FancyInput</code> on the usage site; zero testids inside the
          forwardRef callback (<code>&lt;div&gt;</code>, <code>&lt;label&gt;</code>,{" "}
          <code>&lt;input&gt;</code>)
        </small>
        <FancyInput placeholder="no inner testids" />
      </section>

      <section>
        <h3>React.memo — inner elements have NO testid</h3>
        <small>
          <code>EdgeCasesShowcase.MemoChip</code> on each usage; the inner{" "}
          <code>&lt;span&gt;</code> has no testid
        </small>
        <MemoChip text="Chip A" />
        <MemoChip text="Chip B" />
      </section>

      <section>
        <h3>Switch returns</h3>
        <small>
          Three source-level <code>&lt;div&gt;</code> returns →{" "}
          <code>SwitchDemo.div</code> / <code>SwitchDemo.div2</code> /{" "}
          <code>SwitchDemo.div3</code>
        </small>
        <SwitchDemo status="ok" />
        <SwitchDemo status="warn" />
        <SwitchDemo status="other" />
      </section>

      <section>
        <h3>Counter isolation across components</h3>
        <small>
          <code>IsoAlpha.div</code> + <code>IsoAlpha.button</code>, then{" "}
          <code>IsoBeta.div</code> + <code>IsoBeta.button</code> — NOT{" "}
          <code>IsoBeta.div2</code>
        </small>
        <IsoAlpha />
        <IsoBeta />
      </section>
    </div>
  );
}
