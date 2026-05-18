// async functions are valid React Server Components (Next.js App Router).
// The plugin should tag their JSX just like synchronous components.
async function AsyncPage() {
    const data = await fetchData();
    return <main data-testid="AsyncPage.main">
      <h1 data-testid="AsyncPage.h1">{data.title}</h1>
      <p data-testid="AsyncPage.p">{data.body}</p>
    </main>;
}
const AsyncArrow = async ()=>{
    return <div data-testid="AsyncArrow.div">Loading</div>;
};
// async with early return
async function AsyncWithEarlyReturn({ id }: {
    id: string;
}) {
    if (!id) return <span data-testid="AsyncWithEarlyReturn.span">No ID</span>;
    return <article data-testid="AsyncWithEarlyReturn.article">{id}</article>;
}
