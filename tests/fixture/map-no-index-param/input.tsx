// A callback with no index parameter would give every item the SAME static id.
// The plugin appends an index parameter so each iteration gets a unique testid.
// Array.prototype.map already calls the callback with (item, index), so the added
// parameter is always bound at runtime.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item) => (
      <li>{item}</li>
    ))}
  </ul>
);

// No parameters at all: a placeholder is added so the index can be the 2nd parameter.
const Stars = ({ count }: { count: number }) => (
  <div>
    {Array.from({ length: count }).map(() => (
      <span>*</span>
    ))}
  </div>
);

// A destructured item parameter is left untouched; the index is appended after it.
const Rows = ({ rows }: { rows: { id: string }[] }) => (
  <table>
    <tbody>
      {rows.map(({ id }) => (
        <tr>{id}</tr>
      ))}
    </tbody>
  </table>
);

// function expressions get the same treatment as arrows
const Legacy = ({ items }: { items: string[] }) => (
  <ol>
    {items.map(function (item) {
      return <li>{item}</li>;
    })}
  </ol>
);
