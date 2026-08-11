// `.map(cb, thisArg)` never gets an appended parameter, but an index the source already
// declares is a real binding and is still used for the id.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {items.map(function(item, index) {
        return <li data-testid={"List.li-" + index} key={index}>{item}</li>;
    }, this)}
  </ul>;
