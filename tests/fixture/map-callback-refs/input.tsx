// A callback handed to a map-like call by *reference* has no parameter list at the call
// site, so the index has to be added where the function is declared. The declaration may
// come before or after the call, and a function declaration may be hoisted above it —
// which is why the names are collected in a pass of their own before anything is
// rewritten.
//
// Every segment here is guarded, including the ones the source declares: unlike a
// callback literal, a named function can also be called from somewhere that passes no
// index at all.

// 1. An arrow declared *after* the call site that reaches it.
const List = ({ items }: { items: string[] }) => <ul>{items.map(renderItem)}</ul>;

const renderItem = (item: string) => <li>{item}</li>;

// 2. A hoisted function declaration, with an index it already declares.
const Rows = ({ rows }: { rows: string[] }) => <tbody>{rows.map(renderRow)}</tbody>;

function renderRow(row: string, position: number) {
  return (
    <tr>
      <td>{row}</td>
    </tr>
  );
}

// 3. The collection-first shape passes the reference as argument 1.
const Lodash = ({ items }: { items: string[] }) => (
  <ul>{_.map(items, renderTag)}</ul>
);

function renderTag(tag: string) {
  return <span>{tag}</span>;
}

// 4. A referenced function with no parameters at all still gets an index — in the second
//    slot, behind a placeholder, because that is where a map callee passes it.
const Stars = ({ stars }: { stars: number[] }) => <div>{stars.map(renderStar)}</div>;

const renderStar = () => <i />;

// 5. A referenced function whose JSX is inside its own map keeps both levels.
const Groups = ({ groups }: { groups: string[][] }) => (
  <ul>{groups.map(renderGroup)}</ul>
);

const renderGroup = (group: string[], groupIndex: number) => (
  <li>{group.map((member) => <b>{member}</b>)}</li>
);

// 6. A function that is never referenced as a callback is untouched — no parameter, and
//    a static id.
const plainHelper = (value: string) => <em>{value}</em>;

// 7. A component used as a callback is a callback: `items.map(Row)` calls it with
//    (item, index), so its own elements are numbered per item too.
const Table = ({ cells }: { cells: string[] }) => <table>{cells.map(Cell)}</table>;

const Cell = (props: { value: string }) => <td>{props.value}</td>;
