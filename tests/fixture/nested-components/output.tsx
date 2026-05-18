function Outer() {
    const Inner = ()=><div data-testid="Inner.div">
      <span data-testid="Inner.span">inner</span>
    </div>;
    return <div data-testid="Outer.div">
      <Inner data-testid="Outer.Inner"/>
      <button data-testid="Outer.button">outer</button>
    </div>;
}
