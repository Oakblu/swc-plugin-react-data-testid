// JSX used as a default value in a destructured parameter.
// visit_mut_fn_decl calls visit_mut_children_with AFTER entering the component scope,
// so the default-value JSX is visited while current_component is already set.
// This means it gets tagged — potentially surprising to users.
function WithFallback({ fallback = <Spinner data-testid="WithFallback.Spinner"/> }: {
    fallback?: React.ReactNode;
}) {
    return <main data-testid="WithFallback.main">{fallback}</main>;
}
function WithMultipleDefaults({ header = <DefaultHeader data-testid="WithMultipleDefaults.DefaultHeader"/>, footer = <DefaultFooter data-testid="WithMultipleDefaults.DefaultFooter"/> }: {
    header?: React.ReactNode;
    footer?: React.ReactNode;
}) {
    return <div data-testid="WithMultipleDefaults.div">
      {header}
      <article data-testid="WithMultipleDefaults.article">Content</article>
      {footer}
    </div>;
}
