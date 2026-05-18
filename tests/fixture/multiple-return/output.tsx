const ConditionalComponent = ({ show })=>{
    if (show) {
        return <div data-testid="ConditionalComponent.div">Shown</div>;
    }
    return <span data-testid="ConditionalComponent.span">Hidden</span>;
};
function EarlyReturn({ error }) {
    if (error) return <div data-testid="EarlyReturn.div">Error</div>;
    return <div data-testid="EarlyReturn.div2">
      <p data-testid="EarlyReturn.p">Success</p>
    </div>;
}
