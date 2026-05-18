// An IIFE arrow function is the callee of a CallExpr, not a VarDeclarator init.
// The plugin never opens a new component scope for it; JSX inside shares the
// enclosing component's counter — identical to the .map() callback behaviour.
function Dashboard({ role }: {
    role: string;
}) {
    return <div data-testid="Dashboard.div">
      {(()=>{
        if (role === "admin") return <AdminPanel data-testid="Dashboard.AdminPanel"/>;
        if (role === "mod") return <ModPanel data-testid="Dashboard.ModPanel"/>;
        return <UserPanel data-testid="Dashboard.UserPanel"/>;
    })()}
    </div>;
}
// Nested IIFE inside a conditional.
function Wizard({ step }: {
    step: number;
}) {
    return <section data-testid="Wizard.section">
      {step > 0 && (()=><span data-testid="Wizard.span">Progress: {step}</span>)()}
    </section>;
}
