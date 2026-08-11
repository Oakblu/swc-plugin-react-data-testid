// Second-parameter shapes other than a bare identifier.

// A default value still binds a plain identifier, so the id uses it. The default only
// applies when the callee passes nothing — the same case the guard covers.
const Defaulted = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, index = 0) => <li>{item}</li>)}</ul>
);

// A type annotation is part of the binding, not a different pattern.
const Annotated = ({ items }: { items: string[] }) => (
  <ul>{items.map((item: string, index: number) => <li>{item}</li>)}</ul>
);

// A third parameter is untouched and the second is still the index.
const WithArray = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, index, all) => <li>{all.length}{item}</li>)}</ul>
);

// A destructuring pattern binds no single identifier to reference, so the id stays
// static — the slot is occupied, so nothing is appended either.
const Destructured = ({ items }: { items: string[] }) => (
  <ul>{items.map((item, [i]) => <li>{item}</li>)}</ul>
);

// A default on the *first* parameter still leaves the index slot free. A parameter with
// no default may follow one that has a default — it only changes Function.length.
const DefaultedItem = ({ items }: { items: string[] }) => (
  <ul>{items.map((item = "?") => <li>{item}</li>)}</ul>
);
