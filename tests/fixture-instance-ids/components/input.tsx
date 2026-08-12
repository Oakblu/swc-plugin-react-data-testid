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
const Badge = ({ label }: { label: string }) => <span>{label}</span>;

// 2. Two instances, each handed a different prefix by the parent.
const Toolbar = () => (
  <div>
    <Badge label="a" />
    <Badge label="b" />
  </div>
);

// 3. A props parameter that is a plain identifier is read directly. The `&&` matters:
//    a component function called as a plain helper may get no props at all.
function Card(props: { title: string }) {
  return <article>{props.title}</article>;
}

// 4. A component that declares no parameter gets one, and only because its JSX uses it.
const Divider = () => <hr />;

// 5. A component that already destructures the prop itself is read, not rewritten.
const Titled = ({ "data-testid": testId }: { "data-testid"?: string }) => (
  <h2>{testId}</h2>
);

// 6. Instance prefixes compose with map indices: the child of an iterated element gets
//    both, so every rendered node is distinct.
const Rows = ({ rows }: { rows: string[] }) => (
  <ul>
    {rows.map((row) => (
      <li>
        <Badge label={row} />
      </li>
    ))}
  </ul>
);

// 7. A class component reads this.props.
class Panel extends React.Component {
  render() {
    return <section>{this.props.children}</section>;
  }
}

// 8. …but not from inside a function expression, where `this` is something else. Those
//    elements keep the static component-name prefix.
class Legacy extends React.Component {
  render() {
    return (
      <ul>
        {this.props.items.map(function (item) {
          return <li>{item}</li>;
        })}
      </ul>
    );
  }
}

// 9. An element that already carries the attribute is still never overwritten, and the
//    prefix is not marked used on its account.
const Pinned = () => <div data-testid="pinned" />;

// 10. A rest element keeps collecting everything else — the prefix property is inserted
//     first, because a rest element must stay last. The prop it now captures is one the
//     parent meant for this component, so a `{...rest}` spread no longer forwards it
//     onto the root element and overwrites the id the plugin put there.
const Input = ({ label, ...rest }: { label: string }) => (
  <label>
    {label}
    <input {...rest} />
  </label>
);

// 11. Prefixes chain, so a component three levels down is still addressable by the path
//     it was rendered through.
const Field = () => (
  <div>
    <Input label="name" />
  </div>
);

// 12. A default on the props parameter is transparent — the property is inserted into
//     the pattern underneath it, not around it.
const Defaulted = ({ label } = {}) => <b>{label}</b>;

// 13. An array pattern in the props slot is not a React component, so there is no prefix
//     to read and the ids stay static.
const NotAComponent = ([first]) => <p>{first}</p>;

// 14. A function reached through `items.map(row)` takes an index, never a prefix: its
//     first argument is the item, not props. Reading a prefix off it would be reading a
//     property of the item.
const Listing = ({ items }: { items: string[] }) => <ul>{items.map(renderItem)}</ul>;

const renderItem = (item: string) => <li>{item}</li>;
