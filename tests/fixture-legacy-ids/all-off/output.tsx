// Every opt-out set at once: autoIndex false, collectionFirstMap false, nestedIndexes
// false, mapMethods narrowed to ["map"]. This is the 0.1.x id scheme, for a codebase
// migrating in stages. Nothing here is needed for correctness — the defaults are.
// No index declared: no parameter is appended, so one static id for every item.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {items.map((item)=><li data-testid="List.li">{item}</li>)}
  </ul>;
// flatMap is no longer a recognised name.
const Flat = ({ groups }: {
    groups: string[][];
})=><ul data-testid="Flat.ul">
    {groups.flatMap((group, i)=><li data-testid="Flat.li">{group.join()}</li>)}
  </ul>;
// The collection-first form is no longer recognised.
const Lodash = ({ items }: {
    items: string[];
})=><ul data-testid="Lodash.ul">
    {_.map(items, (item, i)=><li data-testid="Lodash.li">{item}</li>)}
  </ul>;
// Nested callbacks contribute only the innermost index — the 0.1.x id, which repeats
// for every outer row.
const Table = ({ rows }: {
    rows: string[][];
})=><table data-testid="Table.table">
    <tbody data-testid="Table.tbody">
      {rows.map((cells, r)=><tr data-testid={"Table.tr-" + r}>
          {cells.map((cell, c)=><td data-testid={"Table.td-" + c}>{cell}</td>)}
        </tr>)}
    </tbody>
  </table>;
