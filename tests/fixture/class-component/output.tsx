import React from "react";
class Header extends React.Component {
    render() {
        return <header data-testid="Header.header">
        <h1 data-testid="Header.h1">Title</h1>
        <nav data-testid="Header.nav">
          <a data-testid="Header.a" href="/">Home</a>
        </nav>
      </header>;
    }
}
