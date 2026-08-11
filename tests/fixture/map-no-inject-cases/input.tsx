// Shapes where NOTHING dynamic can be emitted: there is no callback body to read an
// index from, or no attribute to write. Every element here keeps a static id.
//
// Shapes where the callee *might* not pass an index are a different case — those are
// inferred with a guarded segment; see fixture map-uncertain-callees.

// 1. A callback passed by reference: there is no parameter list, and any JSX lives in
//    renderItem, which gets ids under its own component name.
const ByReference = ({ items }: { items: string[] }) => (
  <ul>{items.map(renderItem)}</ul>
);

// 2. A callback with no JSX keeps its original signature — no stray parameter.
const NoJsx = ({ items }: { items: string[] }) => (
  <ul>{items.filter((item) => item.length > 0).map((item) => item.trim()).length}</ul>
);

// 3. JSX that already carries the attribute is left alone, so nothing references an
//    index and none is appended.
const AlreadyTagged = ({ items }: { items: string[] }) => (
  <ul>{items.map((item) => <li data-testid="custom">{item}</li>)}</ul>
);

// 4. A destructured second parameter binds no single identifier to reference, and the
//    slot is occupied so nothing can be appended.
const Destructured = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, [i]) => <li>{item}</li>)}</ul>
);
