// Three-level member expression <A.B.C/>: element_name() returns only the last segment ("C").
// The generated testid uses only that last segment, not the full dotted path.
function App() {
    return <Ant.Design.Button data-testid="App.Button">
      Click me
    </Ant.Design.Button>;
}
