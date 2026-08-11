// Helpers that take the collection first and the callback second — lodash's _.map,
// React.Children.map, jQuery.map. The callback is argument 1, not argument 0.
import _ from "lodash";
import { map } from "lodash";
import { Children } from "react";

const Lodash = ({ items }: { items: string[] }) => (
  <ul>{_.map(items, (item, i) => <li key={i}>{item}</li>)}</ul>
);

// No index declared → one is appended to the *second* argument.
const LodashAuto = ({ items }: { items: string[] }) => (
  <ul>{_.map(items, (item) => <li>{item}</li>)}</ul>
);

// Named import of the same function.
const NamedImport = ({ items }: { items: string[] }) => (
  <ul>{map(items, (item) => <li>{item}</li>)}</ul>
);

const ChildrenMap = ({ children }: { children: React.ReactNode }) => (
  <div>{Children.map(children, (child) => <span>{child}</span>)}</div>
);
