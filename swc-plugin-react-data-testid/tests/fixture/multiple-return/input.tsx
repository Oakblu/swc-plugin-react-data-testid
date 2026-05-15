const ConditionalComponent = ({ show }) => {
  if (show) {
    return <div>Shown</div>
  }
  return <span>Hidden</span>
}

function EarlyReturn({ error }) {
  if (error) return <div>Error</div>

  return (
    <div>
      <p>Success</p>
    </div>
  )
}
