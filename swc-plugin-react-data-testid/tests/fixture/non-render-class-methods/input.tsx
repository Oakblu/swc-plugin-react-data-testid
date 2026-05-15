class TestClass extends React.Component {
  componentDidMount() {
    return <div>Not just render</div>
  }

  render() {
    return <div>Should be processed</div>
  }

  helperMethod() {
    return <span>Helper method</span>
  }
}
