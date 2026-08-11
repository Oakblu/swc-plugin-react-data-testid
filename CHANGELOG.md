# Changelog

## 0.2.0

### Unique ids for every iteration form

Elements rendered inside a map callback now get a unique runtime id per item wherever an
index can be reached. The plugin takes the first of these that is available:

| Source | Index comes from |
| ------ | ---------------- |
| `items.map((item, i) => …)` | the declared parameter |
| `items.map((item, i = 0) => …)` | the parameter behind the default |
| `items.map((...args) => …)` | `args[1]` — the signature is left untouched |
| `items.map((item, ...rest) => …)` | `rest[0]` |
| `items.map((item) => …)` | an appended `__testidIdx0` parameter |

and recognises the call shape in every position a callback can occupy:

```jsx
{items.flatMap((item) => <li/>)}       // flatMap, like map
{_.map(items, (item) => <li/>)}        // lodash, Children.map, $.map
{items.map((item) => <li/>, this)}     // .map(cb, thisArg) is still Array.prototype.map
{items$.pipe(map((item) => <li/>))}    // the RxJS operator passes (value, index)
```

Nested callbacks now contribute one id segment each — `Table.td-0-0`, `Table.td-0-1`,
`Table.td-1-0` — where previously only the innermost index was used, so every row
produced the same cell ids.

Where a callee turns out not to pass an index — curried ramda, lodash-fp,
`R.map(fn, list)`, or a helper of your own named `map` — the segment drops out and the id
is the plain static one. Inference is therefore attempted broadly: a wrong guess costs
nothing, while a missed one costs a duplicate id.

### Breaking: some ids change

Only ids that were previously **duplicated** are affected. Update any selector that
targets them:

| Source | 0.1.x | 0.2.0 |
| ------ | ----- | ----- |
| `items.map((item) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `items.flatMap((item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `_.map(items, (item, i) => <li/>)` | `List.li` on every item | `List.li-0`, `List.li-1`, … |
| `rows.map((r, i) => <tr>{r.map((c, j) => <td/>)}</tr>)` | `Table.td-0`, repeated per row | `Table.td-0-0`, `Table.td-0-1`, … |

Each row has its own opt-out, so a large codebase can migrate in stages: row 1
`autoIndex: false`, row 2 `mapMethods: ["map"]`, row 3 `collectionFirstMap: false`,
row 4 `nestedIndexes: false`. Setting all four reproduces the 0.1.x scheme exactly. To pin
an individual id permanently, set the attribute yourself — the plugin never overwrites one
that is already present.

### New options

All default to the behaviour that produces the most unique ids; they exist to turn
inference off, not to switch it on.

- **`autoIndex`** (`boolean`, default `true`) — append an index parameter to map
  callbacks that declare none.
- **`mapMethods`** (`string[]`, default `["map", "flatMap"]`) — callee names treated as
  iteration. Narrow it to leave a name alone, or widen it to add your own iterator.
- **`collectionFirstMap`** (`boolean`, default `true`) — recognise
  `f(collection, callback)`.
- **`nestedIndexes`** (`boolean`, default `true`) — one id segment per enclosing
  callback. Turning it off makes nested ids collide across outer iterations.

### Notes

- An appended index is emitted guarded —
  `{"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}` — so a helper of
  your own named `map` that does not pass an index degrades to the plain static id rather
  than rendering `List.li-undefined`.
- Callbacks that render no JSX keep their original signature, and nothing is appended to a
  callback passed by reference or one whose parameter slots are already full.
- Appending a parameter changes the callback's `Function.length`. Array methods, lodash
  and `React.Children.map` do not branch on callback arity; if something you call does,
  set `autoIndex: false`. Rest-parameter callbacks are read from, never appended to, so
  their arity is unchanged.
- Ids remain per component, not per instance: a component rendered twice repeats its ids.
- A plugin config that cannot be deserialized now **fails the build with an explanatory
  error** instead of being silently ignored. Deserialization is all-or-nothing, so one
  mistyped value (`"autoIndex": "false"`) previously discarded every other option too.

## 0.1.1 – 0.1.2

Static `Component.element` ids, with dynamic ids only for map callbacks that already
declared an index parameter. (0.1.3 was prepared but never published.)
