const Toggle = ({ show }: {
    show: boolean;
})=><div data-testid="Toggle.div">
    {show ? <span data-testid="Toggle.span">Shown</span> : <em data-testid="Toggle.em">Hidden</em>}
  </div>;
