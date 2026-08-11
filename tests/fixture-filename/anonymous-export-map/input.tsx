// An anonymous default export takes its component name from the filename. That name is
// the id prefix, so it has to survive into the dynamic form too.
export default ({ items }: { items: string[] }) => {
  return (
    <ul>
      {items.map((item) => (
        <li>{item}</li>
      ))}
    </ul>
  );
};
