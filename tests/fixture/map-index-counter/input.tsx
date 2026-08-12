// When the callback sits in argument 0 *and* the call has a collection argument after
// it, the plugin can supply an index itself: the appended parameter takes a default that
// reads a counter bound just outside the call. The default is evaluated per invocation
// and only when the callee passed nothing, so the id needs no guard either way.
//
// This is the `R.map(fn, list)` case — ramda calls fn with the element alone, and before
// the counter every item shared one static id.

import * as R from "ramda";

// 1. Ramda order: the counter supplies 0, 1, 2 … and resets each time the call runs.
const Ramda = ({ items }: { items: string[] }) => (
  <ul>{R.map((item) => <li>{item}</li>, items)}</ul>
);

// 2. A curried `map(fn)` gets NO counter. The call evaluates once and the function it
//    returns may be stored and reused, so a counter would keep climbing across every
//    later use instead of tracking item positions. The appended index stays guarded.
const Curried = ({ items }: { items: string[] }) => (
  <ul>{R.map((item) => <li>{item}</li>)(items)}</ul>
);

// 3. An index the source declares is left alone — no counter, no default. Adding one
//    would change what the author's own parameter means.
const Declared = ({ items }: { items: string[] }) => (
  <ul>{R.map((item, i) => <li>{item}</li>, items)}</ul>
);

// 4. A callback with no JSX references nothing, so neither the parameter nor the counter
//    is written back and the call keeps its original shape.
const NoJsx = ({ items }: { items: string[] }) => (
  <ul>{R.map((item) => item.trim(), items).length}</ul>
);

// 5. Nested calls each get their own counter, and the element carries one segment per
//    level.
const Table = ({ rows }: { rows: string[][] }) => (
  <table>{R.map((cells) => <tr>{R.map((cell) => <td>{cell}</td>, cells)}</tr>, rows)}</table>
);

// 6. An awaited argument cannot be wrapped — an arrow may not await — so this falls back
//    to the guarded appended index.
async function Awaited({ load }: { load: () => Promise<string[]> }) {
  return <ul>{R.map((item) => <li>{item}</li>, await load())}</ul>;
}

// 7. The collection-first shape needs no counter: every callee that takes it — lodash,
//    Children.map, jQuery — passes an index or a key.
const Lodash = ({ items }: { items: string[] }) => (
  <ul>{_.map(items, (item) => <li>{item}</li>)}</ul>
);
