// Nested callbacks: the inner index alone repeats for every outer row, so the id must
// carry one segment per enclosing callback — {"Table.td-" + row + "-" + col}.
const Table = ({ rows }: {
    rows: string[][];
})=><table data-testid="Table.table">
    <tbody data-testid="Table.tbody">
      {rows.map((cells, __testidIdx0)=><tr data-testid={"Table.tr-" + __testidIdx0}>
          {cells.map((cell, __testidIdx1)=><td data-testid={"Table.td-" + __testidIdx0 + "-" + __testidIdx1}>{cell}</td>)}
        </tr>)}
    </tbody>
  </table>;
// Mixed: an explicit outer index, an appended inner one.
const Mixed = ({ groups }: {
    groups: string[][];
})=><div data-testid="Mixed.div">
    {groups.map((group, groupIndex)=><section data-testid={"Mixed.section-" + groupIndex} key={groupIndex}>
        {group.map((item, __testidIdx2)=><p data-testid={"Mixed.p-" + groupIndex + "-" + __testidIdx2}>{item}</p>)}
      </section>)}
  </div>;
