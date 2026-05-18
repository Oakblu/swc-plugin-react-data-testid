import React from "react";

// React.forwardRef wraps an arrow — init is Expr::Call, not Arrow/Fn.
// The plugin does not detect this as a component entry point, so no testids are injected.
const MyInput = React.forwardRef<HTMLInputElement>((props, ref) => (
  <div>
    <label>Input</label>
    <input ref={ref} />
  </div>
));
