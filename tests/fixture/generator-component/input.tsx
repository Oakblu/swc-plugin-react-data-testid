// Generator functions are FnDecl nodes; the plugin has no is_generator guard.
// They are not valid React components but will be treated as one — a known
// false positive. This fixture documents that behaviour.

function* ListGenerator() {
  yield <li>Item A</li>;
  yield <li>Item B</li>;
  yield <li>Item C</li>;
}

// A more realistic accidental-generator case: utility that yields JSX rows.
function* TableRows({ data }: { data: string[] }) {
  for (const row of data) {
    yield <tr><td>{row}</td></tr>;
  }
}
