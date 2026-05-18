const Status = ({ code }: {
    code: number;
})=>{
    switch(code){
        case 200:
            return <div data-testid="Status.div">OK</div>;
        case 404:
            return <span data-testid="Status.span">Not Found</span>;
        default:
            return <p data-testid="Status.p">Unknown</p>;
    }
};
