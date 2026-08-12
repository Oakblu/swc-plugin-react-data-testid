// Shapes where the syntax cannot reveal whether the callee passes an index. Detection
// recognises all of them and the guard absorbs the difference: if an index arrives the
// id is unique, and if it does not the segment drops out to the plain static id.
//
// Narrowing detection instead would guarantee duplicate ids for the callees that DO
// pass an index — RxJS's map operator and Array.prototype.map(cb, thisArg) among them.

// 1. Ramda argument order: map(fn, list) calls fn with the element only.
//    Expected at runtime: "Ramda.li" on every item.
import * as R from "ramda";
const Ramda = ({ items }: { items: string[] }) => (
  <ul>{R.map((item) => <li>{item}</li>, items)}</ul>
);

// 2. A bare map(fn): an RxJS operator passes (value, index); curried ramda and lodash-fp
//    pass the value only. Identical syntax, opposite behaviour.
const Curried = ({ items }: { items: string[] }) => (
  <ul>{items.pipe(map((item) => <li>{item}</li>))}</ul>
);

// 3. `.map(cb, thisArg)` IS Array.prototype.map — the callback does get an index.
const ThisArg = ({ items }: { items: string[] }) => (
  <ul>{items.map(function (item) { return <li>{item}</li>; }, this)}</ul>
);

// 4. A rest parameter already collects the index: `args[1]`. Nothing is appended, so the
//    callback's signature is untouched.
const Rest = ({ items }: { items: string[] }) => (
  <ul>{items.map((...args) => <li>{args[0]}</li>)}</ul>
);

// 5. A rest parameter after a named item reads `rest[0]`.
const NamedThenRest = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, ...rest) => <li>{item}</li>)}</ul>
);

// 6. A destructuring pattern in the index slot. The slot is occupied so nothing can be
//    appended, but the pattern does bind something the source pulled out of that
//    argument — the first such binding is used, guarded like any other guess.
const Destructured = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, [i]) => <li>{item}</li>)}</ul>
);

// 7. …including an object pattern, where the binding is usually named outright.
const NamedInPattern = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, { index }) => <li>{item}</li>)}</ul>
);
