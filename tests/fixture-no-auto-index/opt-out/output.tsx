// With autoIndex disabled, callbacks that declare no index keep their signature and
// every item shares one static id — the pre-0.2.0 behaviour.
const List = ({ items }: {
    items: string[];
})=><ul data-testid="List.ul">
    {items.map((item)=><li data-testid="List.li">{item}</li>)}
  </ul>;
// An index the source declares is still used: the option only governs appending one.
const Declared = ({ items }: {
    items: string[];
})=><ul data-testid="Declared.ul">
    {items.map((item, index)=><li data-testid={"Declared.li-" + index} key={index}>{item}</li>)}
  </ul>;
// The option does NOT restore 0.1.x ids for the shapes 0.2.0 newly *detects*. These two
// declare an index, so it is used regardless — 0.1.x emitted a static id here only
// because flatMap and the collection-first form were not recognised at all.
const FlatMap = ({ items }: {
    items: string[];
})=><ul data-testid="FlatMap.ul">
    {items.flatMap((item, i)=><li data-testid={"FlatMap.li-" + i}>{item}</li>)}
  </ul>;
const Lodash = ({ items }: {
    items: string[];
})=><ul data-testid="Lodash.ul">
    {_.map(items, (item, i)=><li data-testid={"Lodash.li-" + i}>{item}</li>)}
  </ul>;
