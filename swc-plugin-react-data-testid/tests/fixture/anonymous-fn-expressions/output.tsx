const obj = {
    render: function() {
        return <div>Anonymous function</div>;
    }
};
const Component = function() {
    return <span data-testid="Component.span">Unnamed function</span>;
};
