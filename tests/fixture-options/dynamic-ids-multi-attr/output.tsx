// Multiple attributes combined with an inferred index. Each attribute needs its OWN copy
// of the index expression — they cannot share one node — and every copy must reference
// the same binding.
// An appended index: both attributes get the guarded form.
const List = ({ items }: {
    items: string[];
})=><ul data-cy="List.ul" data-testid="List.ul">
    {items.map((item, __testidIdx0)=><li data-cy={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)} data-testid={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>)}
  </ul>;
// A declared index: both attributes get the compact unguarded form.
const Declared = ({ items }: {
    items: string[];
})=><ul data-cy="Declared.ul" data-testid="Declared.ul">
    {items.map((item, i)=><li data-cy={"Declared.li-" + i} data-testid={"Declared.li-" + i}>{item}</li>)}
  </ul>;
// One attribute already present inside a map: only the missing one is added, and it is
// still dynamic. The existing value is never overwritten.
const Partial = ({ items }: {
    items: string[];
})=><ul data-cy="Partial.ul" data-testid="Partial.ul">
    {items.map((item, i)=><li data-testid={"Partial.li-" + i} data-cy="pinned">{item}</li>)}
  </ul>;
// Nested callbacks with two attributes: each attribute carries both segments.
const Table = ({ rows }: {
    rows: string[][];
})=><table data-cy="Table.table" data-testid="Table.table">
    <tbody data-cy="Table.tbody" data-testid="Table.tbody">
      {rows.map((cells, __testidIdx1)=><tr data-cy={"Table.tr" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)} data-testid={"Table.tr" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}>
          {cells.map((cell, __testidIdx2)=><td data-cy={"Table.td" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1) + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)} data-testid={"Table.td" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1) + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{cell}</td>)}
        </tr>)}
    </tbody>
  </table>;
