// Counter must increment correctly beyond 3 occurrences.
// Tests that the suffix logic (none, 2, 3, 4, 5...) has no upper bound.
function ItemList() {
    return <ul data-testid="ItemList.ul">
      <li data-testid="ItemList.li">One</li>
      <li data-testid="ItemList.li2">Two</li>
      <li data-testid="ItemList.li3">Three</li>
      <li data-testid="ItemList.li4">Four</li>
      <li data-testid="ItemList.li5">Five</li>
    </ul>;
}
// Mixed: heading tags that share the same counter namespace.
function Article() {
    return <div data-testid="Article.div">
      <p data-testid="Article.p">First</p>
      <p data-testid="Article.p2">Second</p>
      <p data-testid="Article.p3">Third</p>
      <p data-testid="Article.p4">Fourth</p>
      <h2 data-testid="Article.h2">Heading A</h2>
      <p data-testid="Article.p5">Fifth</p>
      <h2 data-testid="Article.h22">Heading B</h2>
    </div>;
}
