const Status = ({ code }: { code: number }) => {
  switch (code) {
    case 200: return <div>OK</div>;
    case 404: return <span>Not Found</span>;
    default: return <p>Unknown</p>;
  }
};
