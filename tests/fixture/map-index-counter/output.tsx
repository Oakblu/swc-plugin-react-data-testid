// When the callback sits in argument 0 *and* the call has a collection argument after
// it, the plugin can supply an index itself: the appended parameter takes a default that
// reads a counter bound just outside the call. The default is evaluated per invocation
// and only when the callee passed nothing, so the id needs no guard either way.
//
// This is the `R.map(fn, list)` case — ramda calls fn with the element alone, and before
// the counter every item shared one static id.
import * as R from "ramda";
// 1. Ramda order: the counter supplies 0, 1, 2 … and resets each time the call runs.
const Ramda = ({ items }: {
    items: string[];
})=><ul data-testid="Ramda.ul">{((__testidSeq0)=>R.map((item, __testidIdx1 = __testidSeq0++)=><li data-testid={"Ramda.li-" + __testidIdx1}>{item}</li>, items))(0)}</ul>;
// 2. A curried `map(fn)` gets NO counter. The call evaluates once and the function it
//    returns may be stored and reused, so a counter would keep climbing across every
//    later use instead of tracking item positions. The appended index stays guarded.
const Curried = ({ items }: {
    items: string[];
})=><ul data-testid="Curried.ul">{R.map((item, __testidIdx2)=><li data-testid={"Curried.li" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{item}</li>)(items)}</ul>;
// 3. An index the source declares is left alone — no counter, no default. Adding one
//    would change what the author's own parameter means.
const Declared = ({ items }: {
    items: string[];
})=><ul data-testid="Declared.ul">{R.map((item, i)=><li data-testid={"Declared.li-" + i}>{item}</li>, items)}</ul>;
// 4. A callback with no JSX references nothing, so neither the parameter nor the counter
//    is written back and the call keeps its original shape.
const NoJsx = ({ items }: {
    items: string[];
})=><ul data-testid="NoJsx.ul">{R.map((item)=>item.trim(), items).length}</ul>;
// 5. Nested calls each get their own counter, and the element carries one segment per
//    level.
const Table = ({ rows }: {
    rows: string[][];
})=><table data-testid="Table.table">{((__testidSeq6)=>R.map((cells, __testidIdx7 = __testidSeq6++)=><tr data-testid={"Table.tr-" + __testidIdx7}>{((__testidSeq8)=>R.map((cell, __testidIdx9 = __testidSeq8++)=><td data-testid={"Table.td-" + __testidIdx7 + "-" + __testidIdx9}>{cell}</td>, cells))(0)}</tr>, rows))(0)}</table>;
// 6. An awaited argument cannot be wrapped — an arrow may not await — so this falls back
//    to the guarded appended index.
async function Awaited({ load }: {
    load: () => Promise<string[]>;
}) {
    return <ul data-testid="Awaited.ul">{R.map((item, __testidIdx10)=><li data-testid={"Awaited.li" + (__testidIdx10 === void 0 ? "" : "-" + __testidIdx10)}>{item}</li>, await load())}</ul>;
}
// 7. The collection-first shape needs no counter: every callee that takes it — lodash,
//    Children.map, jQuery — passes an index or a key.
const Lodash = ({ items }: {
    items: string[];
})=><ul data-testid="Lodash.ul">{_.map(items, (item, __testidIdx11)=><li data-testid={"Lodash.li" + (__testidIdx11 === void 0 ? "" : "-" + __testidIdx11)}>{item}</li>)}</ul>;
