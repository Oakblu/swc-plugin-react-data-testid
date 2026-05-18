// Generator functions are FnDecl nodes; the plugin has no is_generator guard.
// They are not valid React components but will be treated as one — a known
// false positive. This fixture documents that behaviour.
function* ListGenerator() {
    yield <li data-testid="ListGenerator.li">Item A</li>;
    yield <li data-testid="ListGenerator.li2">Item B</li>;
    yield <li data-testid="ListGenerator.li3">Item C</li>;
}
// A more realistic accidental-generator case: utility that yields JSX rows.
function* TableRows({ data }: {
    data: string[];
}) {
    for (const row of data){
        yield <tr data-testid="TableRows.tr"><td data-testid="TableRows.td">{row}</td></tr>;
    }
}
