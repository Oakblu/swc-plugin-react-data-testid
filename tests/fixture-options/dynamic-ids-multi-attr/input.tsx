// Multiple attributes combined with an inferred index. Each attribute needs its OWN copy
// of the index expression — they cannot share one node — and every copy must reference
// the same binding.

// An appended index: both attributes get the guarded form.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item) => (
      <li>{item}</li>
    ))}
  </ul>
);

// A declared index: both attributes get the compact unguarded form.
const Declared = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item, i) => (
      <li>{item}</li>
    ))}
  </ul>
);

// One attribute already present inside a map: only the missing one is added, and it is
// still dynamic. The existing value is never overwritten.
const Partial = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item, i) => (
      <li data-cy="pinned">{item}</li>
    ))}
  </ul>
);

// Nested callbacks with two attributes: each attribute carries both segments.
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
