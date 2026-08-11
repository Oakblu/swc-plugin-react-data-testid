// Index inference inside a class render() method. current_component is activated only
// for render, and the map machinery has to work there exactly as in a function component.
class ItemList extends React.Component {
  render() {
    return (
      <ul>
        {this.props.items.map((item) => (
          <li>{item}</li>
        ))}
      </ul>
    );
  }

  // Not render(): no component is active, so nothing here is touched — including the
  // callback, which must keep its signature.
  renderRow(row: string) {
    return row.split("").map((ch) => <span>{ch}</span>);
  }
}

// A declared index inside render, and a nested callback below it.
class Grid extends React.Component {
  render() {
    return (
      <table>
        <tbody>
          {this.props.rows.map((cells, r) => (
            <tr>
              {cells.map((cell) => (
                <td>{cell}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    );
  }
}
