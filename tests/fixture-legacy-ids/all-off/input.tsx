// Every opt-out set at once: autoIndex false, collectionFirstMap false, nestedIndexes
// false, mapMethods narrowed to ["map"]. This is the 0.1.x id scheme, for a codebase
// migrating in stages. Nothing here is needed for correctness — the defaults are.

// No index declared: no parameter is appended, so one static id for every item.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item) => (
      <li>{item}</li>
    ))}
  </ul>
);

// flatMap is no longer a recognised name.
const Flat = ({ groups }: { groups: string[][] }) => (
  <ul>
    {groups.flatMap((group, i) => (
      <li>{group.join()}</li>
    ))}
  </ul>
);

// The collection-first form is no longer recognised.
const Lodash = ({ items }: { items: string[] }) => (
  <ul>
    {_.map(items, (item, i) => (
      <li>{item}</li>
    ))}
  </ul>
);

// Nested callbacks contribute only the innermost index — the 0.1.x id, which repeats
// for every outer row.
const Table = ({ rows }: { rows: string[][] }) => (
  <table>
    <tbody>
      {rows.map((cells, r) => (
        <tr>
          {cells.map((cell, c) => (
            <td>{cell}</td>
          ))}
        </tr>
      ))}
    </tbody>
  </table>
);
