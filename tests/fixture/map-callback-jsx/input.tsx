// The arrow inside .map() is a call argument, not a var-declarator init.
// The plugin does not open a new component scope for it; current_component stays the enclosing component.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item, index) => (
      <li key={index}>{item}</li>
    ))}
  </ul>
);
