// A callback with no index parameter would give every item the SAME static id.
// The plugin appends an index parameter so each iteration gets a unique testid.
// Array.prototype.map already calls the callback with (item, index), so the added
// parameter is always bound at runtime.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {items.map((item, __testidIdx0)=><li data-testid={"List.li-" + __testidIdx0}>{item}</li>)}
  </ul>;
// No parameters at all: a placeholder is added so the index can be the 2nd parameter.
const Stars = ({ count }: {
    count: number;
})=><div data-testid="Stars.div">
    {Array.from({
        length: count
    }).map((__testidItem2, __testidIdx1)=><span data-testid={"Stars.span-" + __testidIdx1}>*</span>)}
  </div>;
// A destructured item parameter is left untouched; the index is appended after it.
const Rows = ({ rows }: {
    rows: {
        id: string;
    }[];
})=><table data-testid="Rows.table">
    <tbody data-testid="Rows.tbody">
      {rows.map(({ id }, __testidIdx3)=><tr data-testid={"Rows.tr-" + __testidIdx3}>{id}</tr>)}
    </tbody>
  </table>;
// function expressions get the same treatment as arrows
const Legacy = ({ items }: {
    items: string[];
})=><ol data-testid="Legacy.ol">
    {items.map(function(item, __testidIdx4) {
        return <li data-testid={"Legacy.li-" + __testidIdx4}>{item}</li>;
    })}
  </ol>;
