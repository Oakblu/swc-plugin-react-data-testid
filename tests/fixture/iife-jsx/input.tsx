// An IIFE arrow function is the callee of a CallExpr, not a VarDeclarator init.
// The plugin never opens a new component scope for it; JSX inside shares the
// enclosing component's counter — identical to the .map() callback behaviour.

function Dashboard({ role }: { role: string }) {
  return (
    <div>
      {(() => {
        if (role === "admin") return <AdminPanel />;
        if (role === "mod") return <ModPanel />;
        return <UserPanel />;
      })()}
    </div>
  );
}

// Nested IIFE inside a conditional.
function Wizard({ step }: { step: number }) {
  return (
    <section>
      {step > 0 &&
        (() => (
          <span>Progress: {step}</span>
        ))()}
    </section>
  );
}
