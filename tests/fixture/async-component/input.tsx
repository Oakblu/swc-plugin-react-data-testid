// async functions are valid React Server Components (Next.js App Router).
// The plugin should tag their JSX just like synchronous components.

async function AsyncPage() {
  const data = await fetchData();
  return (
    <main>
      <h1>{data.title}</h1>
      <p>{data.body}</p>
    </main>
  );
}

const AsyncArrow = async () => {
  return <div>Loading</div>;
};

// async with early return
async function AsyncWithEarlyReturn({ id }: { id: string }) {
  if (!id) return <span>No ID</span>;
  return <article>{id}</article>;
}
