// JSX inside try/catch/finally blocks must still be tagged.
// This pattern appears in components that parse user input or call risky APIs.

function RiskyComponent() {
  try {
    return <div>Success</div>;
  } catch (error) {
    return <span>Error</span>;
  }
}

function WithFinally() {
  try {
    return <section>Content</section>;
  } catch (e) {
    return <aside>Fallback</aside>;
  } finally {
    cleanup();
  }
}
