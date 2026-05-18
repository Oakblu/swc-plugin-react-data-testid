// Two independent components using the same element types.
// Each component's counter starts at 1 — counters do not bleed across components.
function Alpha() {
    return <div data-testid="Alpha.div">
      <button data-testid="Alpha.button">Alpha</button>
    </div>;
}
function Beta() {
    return <div data-testid="Beta.div">
      <button data-testid="Beta.button">Beta</button>
    </div>;
}
