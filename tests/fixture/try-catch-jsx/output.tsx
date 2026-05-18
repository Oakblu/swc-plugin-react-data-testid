// JSX inside try/catch/finally blocks must still be tagged.
// This pattern appears in components that parse user input or call risky APIs.
function RiskyComponent() {
    try {
        return <div data-testid="RiskyComponent.div">Success</div>;
    } catch (error) {
        return <span data-testid="RiskyComponent.span">Error</span>;
    }
}
function WithFinally() {
    try {
        return <section data-testid="WithFinally.section">Content</section>;
    } catch (e) {
        return <aside data-testid="WithFinally.aside">Fallback</aside>;
    } finally{
        cleanup();
    }
}
