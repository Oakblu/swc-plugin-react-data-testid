// JSX inside useMemo/useCallback is visited while current_component is the enclosing component.
// The inner arrow is not a var-declarator init, so no new component scope is created.
// Result: hook-internal JSX shares the parent component's counter — affecting sibling numbering.
const Widget = () => {
  const memoNode = useMemo(() => <div>Memoized</div>, []);
  return (
    <div>
      {memoNode}
    </div>
  );
};
