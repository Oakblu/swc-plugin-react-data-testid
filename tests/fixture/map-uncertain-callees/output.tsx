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
})=><ul data-testid="Ramda.ul">{((__testidSeq0)=>R.map((item, __testidIdx1 = __testidSeq0++)=><li data-testid={"Ramda.li-" + __testidIdx1}>{item}</li>, items))(0)}</ul>;
// 2. A bare map(fn): an RxJS operator passes (value, index); curried ramda and lodash-fp
//    pass the value only. Identical syntax, opposite behaviour.
const Curried = ({ items }: {
    items: string[];
})=><ul data-testid="Curried.ul">{items.pipe(map((item, __testidIdx2)=><li data-testid={"Curried.li" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{item}</li>))}</ul>;
// 3. `.map(cb, thisArg)` IS Array.prototype.map — the callback does get an index.
const ThisArg = ({ items }: {
    items: string[];
})=><ul data-testid="ThisArg.ul">{((__testidSeq3)=>items.map(function(item, __testidIdx4 = __testidSeq3++) {
            return <li data-testid={"ThisArg.li-" + __testidIdx4}>{item}</li>;
        }, this))(0)}</ul>;
// 4. A rest parameter already collects the index: `args[1]`. Nothing is appended, so the
//    callback's signature is untouched.
const Rest = ({ items }: {
    items: string[];
})=><ul data-testid="Rest.ul">{items.map((...args)=><li data-testid={"Rest.li" + (args[1] === void 0 ? "" : "-" + args[1])}>{args[0]}</li>)}</ul>;
// 5. A rest parameter after a named item reads `rest[0]`.
const NamedThenRest = ({ items }: {
    items: string[];
})=><ul data-testid="NamedThenRest.ul">{items.map((item, ...rest)=><li data-testid={"NamedThenRest.li" + (rest[0] === void 0 ? "" : "-" + rest[0])}>{item}</li>)}</ul>;
// 6. A destructuring pattern in the index slot. The slot is occupied so nothing can be
//    appended, but the pattern does bind something the source pulled out of that
//    argument — the first such binding is used, guarded like any other guess.
const Destructured = ({ items }: {
    items: string[];
})=><ul data-testid="Destructured.ul">{items.map((item, [i])=><li data-testid={"Destructured.li" + (i === void 0 ? "" : "-" + i)}>{item}</li>)}</ul>;
// 7. …including an object pattern, where the binding is usually named outright.
const NamedInPattern = ({ items }: {
    items: string[];
})=><ul data-testid="NamedInPattern.ul">{items.map((item, { index })=><li data-testid={"NamedInPattern.li" + (index === void 0 ? "" : "-" + index)}>{item}</li>)}</ul>;
