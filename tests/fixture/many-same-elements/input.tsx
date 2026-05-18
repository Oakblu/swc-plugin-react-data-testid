// Counter must increment correctly beyond 3 occurrences.
// Tests that the suffix logic (none, 2, 3, 4, 5...) has no upper bound.

function ItemList() {
  return (
    <ul>
      <li>One</li>
      <li>Two</li>
      <li>Three</li>
      <li>Four</li>
      <li>Five</li>
    </ul>
  );
}

// Mixed: heading tags that share the same counter namespace.
function Article() {
  return (
    <div>
      <p>First</p>
      <p>Second</p>
      <p>Third</p>
      <p>Fourth</p>
      <h2>Heading A</h2>
      <p>Fifth</p>
      <h2>Heading B</h2>
    </div>
  );
}
