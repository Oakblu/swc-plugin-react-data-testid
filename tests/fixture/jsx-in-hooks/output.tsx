// JSX inside useMemo/useCallback is visited while current_component is the enclosing component.
// The inner arrow is not a var-declarator init, so no new component scope is created.
// Result: hook-internal JSX shares the parent component's counter — affecting sibling numbering.
const Widget = ()=>{
    const memoNode = useMemo(()=><div data-testid="Widget.div">Memoized</div>, []);
    return <div data-testid="Widget.div2">
      {memoNode}
    </div>;
};
