const Conditional = ({ show }: {
    show: boolean;
})=><section data-testid="Conditional.section">
    {show && <p data-testid="Conditional.p">Only when true</p>}
    <footer data-testid="Conditional.footer">Always visible</footer>
  </section>;
