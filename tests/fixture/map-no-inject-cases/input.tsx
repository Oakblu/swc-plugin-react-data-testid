// Cases where an index parameter must NOT be appended.

// 1. Ramda-style argument order: map(fn, list). The callback is called with the element
//    only, so an appended index would be `undefined` in every id.
import * as R from "ramda";
const Ramda = ({ items }: { items: string[] }) => (
  <ul>{R.map((item) => <li>{item}</li>, items)}</ul>
);

// 2. A bare `map(fn)` — curried ramda / lodash-fp / an RxJS operator, not an iteration
//    over a collection.
const Curried = ({ items }: { items: string[] }) => (
  <ul>{items.pipe(map((item) => <li>{item}</li>))}</ul>
);

// 3. A callback passed by reference: there is no parameter list to extend.
const ByReference = ({ items }: { items: string[] }) => (
  <ul>{items.map(renderItem)}</ul>
);

// 4. A rest parameter: nothing may follow `...args`.
const Rest = ({ items }: { items: string[] }) => (
  <ul>{items.map((...args) => <li>{args[0]}</li>)}</ul>
);

// 5. A callback with no JSX keeps its original signature — no stray parameter.
const NoJsx = ({ items }: { items: string[] }) => (
  <ul>{items.filter((item) => item.length > 0).map((item) => item.trim()).length}</ul>
);

// 6. JSX that already carries the attribute is left alone, so nothing references an
//    index and none is appended.
const AlreadyTagged = ({ items }: { items: string[] }) => (
  <ul>{items.map((item) => <li data-testid="custom">{item}</li>)}</ul>
);

// 7. `.map(cb, thisArg)`: a second argument makes the call shape ambiguous — it is
//    either a thisArg or an unknown callback-first helper — so no parameter is appended.
//    A declared index would still be honoured; see fixture map-this-arg.
const ThisArg = ({ items }: { items: string[] }) => (
  <ul>{items.map(function (item) { return <li>{item}</li>; }, this)}</ul>
);
