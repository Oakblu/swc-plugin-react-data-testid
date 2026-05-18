import React from "react";
// React.memo wraps an arrow — same as forwardRef, init is Expr::Call.
// The plugin does not detect this as a component entry point, so no testids are injected.
const Button = React.memo(({ onClick }: {
    onClick: () => void;
})=><button onClick={onClick}>Click me</button>);
