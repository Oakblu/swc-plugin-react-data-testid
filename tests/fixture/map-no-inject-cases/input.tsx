// Shapes where NOTHING dynamic can be emitted: there is nothing in the file to read an
// index from, or no attribute to write. Every element here keeps a static id.
//
// Shapes where the callee *might* not pass an index are a different case — those are
// inferred with a guarded segment; see fixture map-uncertain-callees.

// 1. A callback passed by reference to a function this file does not declare. The
//    declaration is where an index would be added (see fixture map-callback-refs), and
//    an imported one is out of reach — so this call site is left alone entirely.
import { renderItem } from "./render-item";

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
