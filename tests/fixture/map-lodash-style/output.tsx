// Helpers that take the collection first and the callback second — lodash's _.map,
// React.Children.map, jQuery.map. The callback is argument 1, not argument 0.
import _ from "lodash";
import { map } from "lodash";
import { Children } from "react";
const Lodash = ({ items }: {
    items: string[];
})=><ul data-testid="Lodash.ul">{_.map(items, (item, i)=><li data-testid={"Lodash.li-" + i} key={i}>{item}</li>)}</ul>;
// No index declared → one is appended to the *second* argument.
const LodashAuto = ({ items }: {
    items: string[];
})=><ul data-testid="LodashAuto.ul">{_.map(items, (item, __testidIdx0)=><li data-testid={"LodashAuto.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>)}</ul>;
// Named import of the same function.
const NamedImport = ({ items }: {
    items: string[];
})=><ul data-testid="NamedImport.ul">{map(items, (item, __testidIdx1)=><li data-testid={"NamedImport.li" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}>{item}</li>)}</ul>;
const ChildrenMap = ({ children }: {
    children: React.ReactNode;
})=><div data-testid="ChildrenMap.div">{Children.map(children, (child, __testidIdx2)=><span data-testid={"ChildrenMap.span" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{child}</span>)}</div>;
