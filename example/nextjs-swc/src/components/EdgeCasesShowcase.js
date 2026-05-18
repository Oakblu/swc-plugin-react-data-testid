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

// ─── Many identical elements ──────────────────────────────────────────────────
// Counter suffix is appended only from the second occurrence:
// li → li2 → li3 → li4 → li5
function ManyItemsList() {
  return (
    <ul>
      <li>One (ManyItemsList.li)</li>
      <li>Two (ManyItemsList.li2)</li>
      <li>Three (ManyItemsList.li3)</li>
      <li>Four (ManyItemsList.li4)</li>
      <li>Five (ManyItemsList.li5)</li>
    </ul>
  );
}

// ─── IIFE counter sharing ─────────────────────────────────────────────────────
// The IIFE arrow is a CallExpr callee, not a VarDeclarator init.
// No new scope is opened — JSX inside shares the enclosing component's counter.
// Expected: IifeDemo.div, then IifeDemo.strong or IifeDemo.em
function IifeDemo({ role }) {
  return (
    <div>
      {(() => {
        if (role === "admin") return <strong>Admin (IifeDemo.strong)</strong>;
        return <em>Guest (IifeDemo.em)</em>;
      })()}
    </div>
  );
}

// ─── Try / catch JSX ─────────────────────────────────────────────────────────
// JSX in both the try block and the catch block is tagged.
// Expected: TryCatchDemo.div + TryCatchDemo.p (happy path)
//        or TryCatchDemo.span                  (error path)
function TryCatchDemo() {
  const [hasError, setHasError] = useState(false);
  try {
    if (hasError) throw new Error("demo error");
    return (
      <div>
        <p>No error (TryCatchDemo.div + TryCatchDemo.p)</p>
        <button onClick={() => setHasError(true)}>Trigger catch branch</button>
      </div>
    );
  } catch (e) {
    return (
      <span>
        Caught: {e.message} (TryCatchDemo.span){" "}
        <button onClick={() => setHasError(false)}>Reset</button>
      </span>
    );
  }
}

// ─── JSX in default params ────────────────────────────────────────────────────
// enter_component runs before visit_mut_children_with, so param default-value
// JSX is visited while current_component is already set.
// Expected: DefaultParamDemo.em  (the default-param JSX, visited before the body!)
//         + DefaultParamDemo.div  (the body JSX)
function DefaultParamDemo({ fallback = <em>default fallback</em> }) {
  return (
    <div>
      {fallback}
    </div>
  );
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

      <section>
        <h3>Many identical elements — counter past 3</h3>
        <small>
          Suffix sequence: no suffix → 2 → 3 → 4 → 5. Inspect each{" "}
          <code>&lt;li&gt;</code> to verify.
        </small>
        <ManyItemsList />
      </section>

      <section>
        <h3>IIFE counter sharing</h3>
        <small>
          The IIFE arrow is a <code>CallExpr</code> callee, not a{" "}
          <code>VarDeclarator</code> init — no new scope.{" "}
          <code>IifeDemo.strong</code> / <code>IifeDemo.em</code> share the
          parent counter alongside <code>IifeDemo.div</code>.
        </small>
        <IifeDemo role="admin" />
        <IifeDemo role="guest" />
      </section>

      <section>
        <h3>Try / catch JSX</h3>
        <small>
          Both branches are tagged. Happy path:{" "}
          <code>TryCatchDemo.div</code> + <code>TryCatchDemo.p</code>.{" "}
          Error path: <code>TryCatchDemo.span</code>.
        </small>
        <TryCatchDemo />
      </section>

      <section>
        <h3>JSX in default params</h3>
        <small>
          The plugin enters component scope before visiting params, so the{" "}
          <code>&lt;em&gt;</code> in the default value gets{" "}
          <code>DefaultParamDemo.em</code> — before body elements.
          The wrapping <code>&lt;div&gt;</code> is{" "}
          <code>DefaultParamDemo.div</code>.
        </small>
        <DefaultParamDemo />
      </section>
    </div>
  );
}
