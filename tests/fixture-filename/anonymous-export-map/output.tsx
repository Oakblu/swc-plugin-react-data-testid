// An anonymous default export takes its component name from the filename. That name is
// the id prefix, so it has to survive into the dynamic form too.
export default (({ items }: {
    items: string[];
})=>{
    return <ul data-testid="Star-list.ul">
      {items.map((item, __testidIdx0)=><li data-testid={"Star-list.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>)}
    </ul>;
});
