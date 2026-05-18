function Outer() {
  const Inner = () => (
    <div>
      <span>inner</span>
    </div>
  );
  return (
    <div>
      <Inner />
      <button>outer</button>
    </div>
  );
}
