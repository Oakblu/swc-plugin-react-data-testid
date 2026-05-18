const Conditional = ({ show }: { show: boolean }) => (
  <section>
    {show && <p>Only when true</p>}
    <footer>Always visible</footer>
  </section>
);
