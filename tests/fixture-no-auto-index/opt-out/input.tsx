// With autoIndex disabled, callbacks that declare no index keep their signature and
// every item shares one static id — the pre-0.2.0 behaviour.
const List = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item) => (
      <li>{item}</li>
    ))}
  </ul>
);

// An index the source declares is still used: the option only governs appending one.
const Declared = ({ items }: { items: string[] }) => (
  <ul>
    {items.map((item, index) => (
      <li key={index}>{item}</li>
    ))}
  </ul>
);

// The option does NOT restore 0.1.x ids for the shapes 0.2.0 newly *detects*. These two
// declare an index, so it is used regardless — 0.1.x emitted a static id here only
// because flatMap and the collection-first form were not recognised at all.
const FlatMap = ({ items }: { items: string[] }) => (
  <ul>
    {items.flatMap((item, i) => (
      <li>{item}</li>
    ))}
  </ul>
);

const Lodash = ({ items }: { items: string[] }) => (
  <ul>
    {_.map(items, (item, i) => (
      <li>{item}</li>
    ))}
  </ul>
);
