// flatMap passes (item, index) exactly like map, both with an explicit index parameter
// and with one appended by the plugin.
const Groups = ({ groups }: {
    groups: {
        rows: string[];
    }[];
})=><ul data-testid="Groups.ul">
    {groups.flatMap((group, i)=><li data-testid={"Groups.li-" + i} key={i}>{group.rows.length}</li>)}
  </ul>;
const Tags = ({ tags }: {
    tags: string[][];
})=><div data-testid="Tags.div">
    {tags.flatMap((group, __testidIdx0)=><span data-testid={"Tags.span-" + __testidIdx0}>{group.length}</span>)}
  </div>;
