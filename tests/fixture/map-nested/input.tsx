// Nested callbacks: the inner index alone repeats for every outer row, so the id must
// carry one segment per enclosing callback — {"Table.td-" + row + "-" + col}.
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

// Mixed: an explicit outer index, an appended inner one.
const Mixed = ({ groups }: { groups: string[][] }) => (
  <div>
    {groups.map((group, groupIndex) => (
      <section key={groupIndex}>
        {group.map((item) => (
          <p>{item}</p>
        ))}
      </section>
    ))}
  </div>
);
