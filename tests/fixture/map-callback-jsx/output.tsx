// The arrow inside .map() is a call argument, not a var-declarator init.
// The plugin does not open a new component scope; current_component stays the enclosing component.
// When the callback declares an index parameter, the testid becomes dynamic:
//   data-testid={"List.li-" + index}  (unique per item at runtime)
// Without an index parameter, the testid would be the static "List.li" for every item.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {items.map((item, index)=><li data-testid={"List.li-" + index} key={index}>{item}</li>)}
  </ul>;
