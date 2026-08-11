// flatMap passes (item, index) exactly like map, both with an explicit index parameter
// and with one appended by the plugin.
const Groups = ({ groups }: { groups: { rows: string[] }[] }) => (
  <ul>
    {groups.flatMap((group, i) => (
      <li key={i}>{group.rows.length}</li>
    ))}
  </ul>
);

const Tags = ({ tags }: { tags: string[][] }) => (
  <div>
    {tags.flatMap((group) => (
      <span>{group.length}</span>
    ))}
  </div>
);
