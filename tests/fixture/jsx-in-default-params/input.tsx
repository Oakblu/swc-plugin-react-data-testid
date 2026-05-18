// JSX used as a default value in a destructured parameter.
// visit_mut_fn_decl calls visit_mut_children_with AFTER entering the component scope,
// so the default-value JSX is visited while current_component is already set.
// This means it gets tagged — potentially surprising to users.

function WithFallback({ fallback = <Spinner /> }: { fallback?: React.ReactNode }) {
  return <main>{fallback}</main>;
}

function WithMultipleDefaults({
  header = <DefaultHeader />,
  footer = <DefaultFooter />,
}: {
  header?: React.ReactNode;
  footer?: React.ReactNode;
}) {
  return (
    <div>
      {header}
      <article>Content</article>
      {footer}
    </div>
  );
}
