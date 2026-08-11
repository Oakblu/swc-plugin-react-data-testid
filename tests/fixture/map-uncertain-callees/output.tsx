// Shapes where the syntax cannot reveal whether the callee passes an index. Detection
// recognises all of them and the guard absorbs the difference: if an index arrives the
// id is unique, and if it does not the segment drops out to the plain static id.
//
// Narrowing detection instead would guarantee duplicate ids for the callees that DO
// pass an index — RxJS's map operator and Array.prototype.map(cb, thisArg) among them.
// 1. Ramda argument order: map(fn, list) calls fn with the element only.
//    Expected at runtime: "Ramda.li" on every item.
import * as R from "ramda";
const Ramda = ({ items }: {
    items: string[];
})=><ul data-testid="Ramda.ul">{R.map((item, __testidIdx0)=><li data-testid={"Ramda.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>, items)}</ul>;
// 2. A bare map(fn): an RxJS operator passes (value, index); curried ramda and lodash-fp
//    pass the value only. Identical syntax, opposite behaviour.
const Curried = ({ items }: {
    items: string[];
})=><ul data-testid="Curried.ul">{items.pipe(map((item, __testidIdx1)=><li data-testid={"Curried.li" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}>{item}</li>))}</ul>;
// 3. `.map(cb, thisArg)` IS Array.prototype.map — the callback does get an index.
const ThisArg = ({ items }: {
    items: string[];
})=><ul data-testid="ThisArg.ul">{items.map(function(item, __testidIdx2) {
        return <li data-testid={"ThisArg.li" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{item}</li>;
    }, this)}</ul>;
// 4. A rest parameter already collects the index: `args[1]`. Nothing is appended, so the
//    callback's signature is untouched.
const Rest = ({ items }: {
    items: string[];
})=><ul data-testid="Rest.ul">{items.map((...args)=><li data-testid={"Rest.li" + (args[1] === void 0 ? "" : "-" + args[1])}>{args[0]}</li>)}</ul>;
// 5. A rest parameter after a named item reads `rest[0]`.
const NamedThenRest = ({ items }: {
    items: string[];
})=><ul data-testid="NamedThenRest.ul">{items.map((item, ...rest)=><li data-testid={"NamedThenRest.li" + (rest[0] === void 0 ? "" : "-" + rest[0])}>{item}</li>)}</ul>;
