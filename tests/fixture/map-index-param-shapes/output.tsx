// Second-parameter shapes other than a bare identifier.
// A default value still binds a plain identifier, so the id uses it. The default only
// applies when the callee passes nothing — the same case the guard covers.
const Defaulted = ({ items }: {
    items: string[];
})=><ul data-testid="Defaulted.ul">{items.map((item, index = 0)=><li data-testid={"Defaulted.li-" + index}>{item}</li>)}</ul>;
// A type annotation is part of the binding, not a different pattern.
const Annotated = ({ items }: {
    items: string[];
})=><ul data-testid="Annotated.ul">{items.map((item: string, index: number)=><li data-testid={"Annotated.li-" + index}>{item}</li>)}</ul>;
// A third parameter is untouched and the second is still the index.
const WithArray = ({ items }: {
    items: string[];
})=><ul data-testid="WithArray.ul">{items.map((item, index, all)=><li data-testid={"WithArray.li-" + index}>{all.length}{item}</li>)}</ul>;
// A destructuring pattern binds no single identifier to reference, so the id stays
// static — the slot is occupied, so nothing is appended either.
const Destructured = ({ items }: {
    items: string[];
})=><ul data-testid="Destructured.ul">{items.map((item, [i])=><li data-testid="Destructured.li">{item}</li>)}</ul>;
