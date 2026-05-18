const Toggle = ({ show }: { show: boolean }) => (
  <div>
    {show ? <span>Shown</span> : <em>Hidden</em>}
  </div>
);
