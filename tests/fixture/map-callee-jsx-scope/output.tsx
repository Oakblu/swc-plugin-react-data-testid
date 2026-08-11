// JSX appearing in the .map() *callee* (or in a later argument) is outside the callback,
// so the index binding does not exist there. Those elements must get a STATIC testid.
// Only JSX inside the callback may use the dynamic {"...-" + index} form.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {[
        <Header data-testid="List.Header" key="h"/>,
        ...items
    ].map((item, index)=><li data-testid={"List.li-" + index} key={index}>{item}</li>)}
  </ul>;
