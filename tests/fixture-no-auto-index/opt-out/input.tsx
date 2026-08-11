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
