// The arrow inside .map() is a call argument, not a var-declarator init.
// The plugin does not open a new component scope; current_component stays the enclosing component.
// When the callback declares an index parameter, the testid becomes dynamic:
//   data-testid={"List.li-" + index}  (unique per item at runtime)
// A callback that declares no index gets one appended — see map-no-index-param.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item, index) => (
      <li key={index}>{item}</li>
    ))}
  </ul>
);
