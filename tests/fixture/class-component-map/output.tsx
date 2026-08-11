// Index inference inside a class render() method. current_component is activated only
// for render, and the map machinery has to work there exactly as in a function component.
class ItemList extends React.Component {
    render() {
        return <ul data-testid="ItemList.ul">
        {this.props.items.map((item, __testidIdx0)=><li data-testid={"ItemList.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}>{item}</li>)}
      </ul>;
    }
    // Not render(): no component is active, so nothing here is touched — including the
    // callback, which must keep its signature.
    renderRow(row: string) {
        return row.split("").map((ch)=><span>{ch}</span>);
    }
}
// A declared index inside render, and a nested callback below it.
class Grid extends React.Component {
    render() {
        return <table data-testid="Grid.table">
        <tbody data-testid="Grid.tbody">
          {this.props.rows.map((cells, r)=><tr data-testid={"Grid.tr-" + r}>
              {cells.map((cell, __testidIdx2)=><td data-testid={"Grid.td-" + r + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>{cell}</td>)}
            </tr>)}
        </tbody>
      </table>;
    }
}
