import React, { Component } from "react";

class ClassComponent extends Component {
  constructor(props) {
    super(props);
    this.state = {
      count: 0,
      message: "Hello from Class Component!",
    };
  }

  handleIncrement = () => {
    this.setState({ count: this.state.count + 1 });
  };

  handleDecrement = () => {
    this.setState({ count: this.state.count - 1 });
  };

  render() {
    return (
      <div>
        <h3>Class Component Example</h3>
        <div>
          <p>{this.state.message}</p>
          <div>
            <span>Count: {this.state.count}</span>
          </div>
          <div>
            <button onClick={this.handleDecrement}>-</button>
            <button onClick={this.handleIncrement}>+</button>
          </div>
        </div>
      </div>
    );
  }
}

export default ClassComponent;
