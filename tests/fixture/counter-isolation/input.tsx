// Two independent components using the same element types.
// Each component's counter starts at 1 — counters do not bleed across components.
function Alpha() {
  return (
    <div>
      <button>Alpha</button>
    </div>
  );
}

function Beta() {
  return (
    <div>
      <button>Beta</button>
    </div>
  );
}
