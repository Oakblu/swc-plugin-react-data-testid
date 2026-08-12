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
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">{items.map(renderItem)}</ul>;
const renderItem = (item: string, __testidIdx0)=><li data-testid={"renderItem.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>;
// 2. A hoisted function declaration, with an index it already declares.
const Rows = ({ rows }: {
    rows: string[];
})=><tbody data-testid="Rows.tbody">{rows.map(renderRow)}</tbody>;
function renderRow(row: string, position: number) {
    return <tr data-testid={"renderRow.tr" + (position === void 0 ? "" : "-" + position)}>
      <td data-testid={"renderRow.td" + (position === void 0 ? "" : "-" + position)}>{row}</td>
    </tr>;
}
// 3. The collection-first shape passes the reference as argument 1.
const Lodash = ({ items }: {
    items: string[];
})=><ul data-testid="Lodash.ul">{_.map(items, renderTag)}</ul>;
function renderTag(tag: string, __testidIdx1) {
    return <span data-testid={"renderTag.span" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}>{tag}</span>;
}
// 4. A referenced function with no parameters at all still gets an index — in the second
//    slot, behind a placeholder, because that is where a map callee passes it.
const Stars = ({ stars }: {
    stars: number[];
})=><div data-testid="Stars.div">{stars.map(renderStar)}</div>;
const renderStar = (__testidItem3, __testidIdx2)=><i data-testid={"renderStar.i" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}/>;
// 5. A referenced function whose JSX is inside its own map keeps both levels.
const Groups = ({ groups }: {
    groups: string[][];
})=><ul data-testid="Groups.ul">{groups.map(renderGroup)}</ul>;
const renderGroup = (group: string[], groupIndex: number)=><li data-testid={"renderGroup.li" + (groupIndex === void 0 ? "" : "-" + groupIndex)}>{group.map((member, __testidIdx4)=><b data-testid={"renderGroup.b" + (groupIndex === void 0 ? "" : "-" + groupIndex) + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}>{member}</b>)}</li>;
// 6. A function that is never referenced as a callback is untouched — no parameter, and
//    a static id.
const plainHelper = (value: string)=><em data-testid="plainHelper.em">{value}</em>;
// 7. A component used as a callback is a callback: `items.map(Row)` calls it with
//    (item, index), so its own elements are numbered per item too.
const Table = ({ cells }: {
    cells: string[];
})=><table data-testid="Table.table">{cells.map(Cell)}</table>;
const Cell = (props: {
    value: string;
}, __testidIdx5)=><td data-testid={"Cell.td" + (__testidIdx5 === void 0 ? "" : "-" + __testidIdx5)}>{props.value}</td>;
