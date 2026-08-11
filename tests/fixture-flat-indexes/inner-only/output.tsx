// nestedIndexes: false with autoIndex still ON — the combination where only the
// innermost callback contributes a segment, but parameters are still being appended.
// The outer callback's index is never referenced, so no parameter may be appended to it.
const Table = ({ rows }: {
    rows: string[][];
})=><table data-testid="Table.table">
    <tbody data-testid="Table.tbody">
      {rows.map((cells, __testidIdx0)=><tr data-testid={"Table.tr" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>
          {cells.map((cell, __testidIdx1)=><td data-testid={"Table.td" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}>{cell}</td>)}
        </tr>)}
    </tbody>
  </table>;
// The outer element itself is inside only one callback, so it still gets that index.
const Outer = ({ groups }: {
    groups: string[][];
})=><div data-testid="Outer.div">
    {groups.map((group, __testidIdx2)=><section data-testid={"Outer.section" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{group.length}</section>)}
  </div>;
// The outer callback renders no JSX of its own, so with nestedIndexes off nothing ever
// references its index — it must keep its original signature.
const Flattened = ({ rows }: {
    rows: string[][];
})=><div data-testid="Flattened.div">{rows.map((cells)=>cells.map((cell, __testidIdx4)=><span data-testid={"Flattened.span" + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}>{cell}</span>))}</div>;
