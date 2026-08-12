// instanceIds: true — a component prefixes its ids with the `data-testid` it was handed,
// so two instances of the same component no longer repeat ids. The prop is the attribute
// the plugin already injects on the child element, so nothing has to be wired up:
//
//   <Badge/> <Badge/>  ->  "List.Badge.span" and "List.Badge2.span"
//
// The fallback is the component's own name, which is exactly the id it had before, so a
// component rendered at the top level (or by code the plugin never saw) is unchanged.
// 1. A destructured props parameter gains one property, inserted first so a rest element
//    stays last.
const Badge = ({ "data-testid": __testidPrefix0, label }: {
    label: string;
})=><span data-testid={(__testidPrefix0 || "Badge") + ".span"}>{label}</span>;
// 2. Two instances, each handed a different prefix by the parent.
const Toolbar = ({ "data-testid": __testidPrefix1 } = {})=><div data-testid={(__testidPrefix1 || "Toolbar") + ".div"}>
    <Badge data-testid={(__testidPrefix1 || "Toolbar") + ".Badge"} label="a"/>
    <Badge data-testid={(__testidPrefix1 || "Toolbar") + ".Badge2"} label="b"/>
  </div>;
// 3. A props parameter that is a plain identifier is read directly. The `&&` matters:
//    a component function called as a plain helper may get no props at all.
function Card(props: {
    title: string;
}) {
    return <article data-testid={(props && props["data-testid"] || "Card") + ".article"}>{props.title}</article>;
}
// 4. A component that declares no parameter gets one, and only because its JSX uses it.
const Divider = ({ "data-testid": __testidPrefix2 } = {})=><hr data-testid={(__testidPrefix2 || "Divider") + ".hr"}/>;
// 5. A component that already destructures the prop itself is read, not rewritten.
const Titled = ({ "data-testid": testId }: {
    "data-testid"?: string;
})=><h2 data-testid={(testId || "Titled") + ".h2"}>{testId}</h2>;
// 6. Instance prefixes compose with map indices: the child of an iterated element gets
//    both, so every rendered node is distinct.
const Rows = ({ "data-testid": __testidPrefix3, rows }: {
    rows: string[];
})=><ul data-testid={(__testidPrefix3 || "Rows") + ".ul"}>
    {rows.map((row, __testidIdx4)=><li data-testid={(__testidPrefix3 || "Rows") + ".li" + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}>
        <Badge data-testid={(__testidPrefix3 || "Rows") + ".Badge" + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)} label={row}/>
      </li>)}
  </ul>;
// 7. A class component reads this.props.
class Panel extends React.Component {
    render() {
        return <section data-testid={(this.props && this.props["data-testid"] || "Panel") + ".section"}>{this.props.children}</section>;
    }
}
// 8. …but not from inside a function expression, where `this` is something else. Those
//    elements keep the static component-name prefix.
class Legacy extends React.Component {
    render() {
        return <ul data-testid={(this.props && this.props["data-testid"] || "Legacy") + ".ul"}>
        {this.props.items.map(function(item, __testidIdx5) {
            return <li data-testid={"Legacy.li" + (__testidIdx5 === void 0 ? "" : "-" + __testidIdx5)}>{item}</li>;
        })}
      </ul>;
    }
}
// 9. An element that already carries the attribute is still never overwritten, and the
//    prefix is not marked used on its account.
const Pinned = ()=><div data-testid="pinned"/>;
// 10. A rest element keeps collecting everything else — the prefix property is inserted
//     first, because a rest element must stay last. The prop it now captures is one the
//     parent meant for this component, so a `{...rest}` spread no longer forwards it
//     onto the root element and overwrites the id the plugin put there.
const Input = ({ "data-testid": __testidPrefix7, label, ...rest }: {
    label: string;
})=><label data-testid={(__testidPrefix7 || "Input") + ".label"}>
    {label}
    <input data-testid={(__testidPrefix7 || "Input") + ".input"} {...rest}/>
  </label>;
// 11. Prefixes chain, so a component three levels down is still addressable by the path
//     it was rendered through.
const Field = ({ "data-testid": __testidPrefix8 } = {})=><div data-testid={(__testidPrefix8 || "Field") + ".div"}>
    <Input data-testid={(__testidPrefix8 || "Field") + ".Input"} label="name"/>
  </div>;
// 12. A default on the props parameter is transparent — the property is inserted into
//     the pattern underneath it, not around it.
const Defaulted = ({ "data-testid": __testidPrefix9, label } = {})=><b data-testid={(__testidPrefix9 || "Defaulted") + ".b"}>{label}</b>;
// 13. An array pattern in the props slot is not a React component, so there is no prefix
//     to read and the ids stay static.
const NotAComponent = ([first])=><p data-testid="NotAComponent.p">{first}</p>;
// 14. A function reached through `items.map(row)` takes an index, never a prefix: its
//     first argument is the item, not props. Reading a prefix off it would be reading a
//     property of the item.
const Listing = ({ "data-testid": __testidPrefix10, items }: {
    items: string[];
})=><ul data-testid={(__testidPrefix10 || "Listing") + ".ul"}>{items.map(renderItem)}</ul>;
const renderItem = (item: string, __testidIdx11)=><li data-testid={"renderItem.li" + (__testidIdx11 === void 0 ? "" : "-" + __testidIdx11)}>{item}</li>;
