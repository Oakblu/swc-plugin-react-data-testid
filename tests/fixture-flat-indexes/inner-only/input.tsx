// nestedIndexes: false with autoIndex still ON — the combination where only the
// innermost callback contributes a segment, but parameters are still being appended.
// The outer callback's index is never referenced, so no parameter may be appended to it.
const Table = ({ rows }: { rows: string[][] }) => (
  <table>
    <tbody>
      {rows.map((cells) => (
        <tr>
          {cells.map((cell) => (
            <td>{cell}</td>
          ))}
        </tr>
      ))}
    </tbody>
  </table>
);

// The outer element itself is inside only one callback, so it still gets that index.
const Outer = ({ groups }: { groups: string[][] }) => (
  <div>
    {groups.map((group) => (
      <section>{group.length}</section>
    ))}
  </div>
);

// The outer callback renders no JSX of its own, so with nestedIndexes off nothing ever
// references its index — it must keep its original signature.
const Flattened = ({ rows }: { rows: string[][] }) => (
  <div>{rows.map((cells) => cells.map((cell) => <span>{cell}</span>))}</div>
);
