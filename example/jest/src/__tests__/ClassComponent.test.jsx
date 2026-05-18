import "@testing-library/jest-dom";
import React from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ClassComponent from "../components/ClassComponent";

describe("ClassComponent — data-testid generation", () => {
  it("injects testids via the render() method scope", () => {
    render(<ClassComponent />);

    // div elements — four nested divs
    expect(screen.getByTestId("ClassComponent.div")).toBeInTheDocument();
    expect(screen.getByTestId("ClassComponent.div2")).toBeInTheDocument();
    expect(screen.getByTestId("ClassComponent.div3")).toBeInTheDocument();
    expect(screen.getByTestId("ClassComponent.div4")).toBeInTheDocument();

    // Text elements
    expect(screen.getByTestId("ClassComponent.h3")).toHaveTextContent("Class Component Example");
    expect(screen.getByTestId("ClassComponent.p")).toHaveTextContent("Hello from Class Component!");
    expect(screen.getByTestId("ClassComponent.span")).toHaveTextContent("Count: 0");

    // Buttons with sequential counter
    expect(screen.getByTestId("ClassComponent.button")).toHaveTextContent("-");
    expect(screen.getByTestId("ClassComponent.button2")).toHaveTextContent("+");
  });

  it("increment and decrement update state correctly", async () => {
    const user = userEvent.setup();
    render(<ClassComponent />);

    const incrementBtn = screen.getByTestId("ClassComponent.button2");
    const countDisplay = screen.getByTestId("ClassComponent.span");

    await user.click(incrementBtn);
    expect(countDisplay).toHaveTextContent("Count: 1");

    await user.click(incrementBtn);
    expect(countDisplay).toHaveTextContent("Count: 2");

    const decrementBtn = screen.getByTestId("ClassComponent.button");
    await user.click(decrementBtn);
    expect(countDisplay).toHaveTextContent("Count: 1");
  });
});
