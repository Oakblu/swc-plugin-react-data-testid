import React from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Button, IconButton, LinkButton } from "../components/Buttons";

describe("Button — data-testid generation", () => {
  it("adds a testid to the single button element", () => {
    render(<Button>Click me</Button>);
    expect(screen.getByTestId("Button.button")).toHaveTextContent("Click me");
  });

  it("forwards click events", async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Save</Button>);

    await user.click(screen.getByTestId("Button.button"));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("respects the disabled prop", () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByTestId("Button.button")).toBeDisabled();
  });
});

describe("IconButton — data-testid generation", () => {
  it("adds testids to button and both spans", () => {
    render(<IconButton icon="★" onClick={vi.fn()}>Star</IconButton>);

    expect(screen.getByTestId("IconButton.button")).toBeInTheDocument();
    expect(screen.getByTestId("IconButton.span")).toHaveTextContent("★");
    expect(screen.getByTestId("IconButton.span2")).toHaveTextContent("Star");
  });

  it("counter is isolated from Button", () => {
    const { container } = render(
      <div>
        <Button>B</Button>
        <IconButton icon="x" onClick={vi.fn()}>I</IconButton>
      </div>
    );

    expect(container.querySelector('[data-testid="Button.button"]')).toBeInTheDocument();
    expect(container.querySelector('[data-testid="IconButton.button"]')).toBeInTheDocument();
  });
});

describe("LinkButton — data-testid generation", () => {
  it("adds testids to both anchor and nested button", () => {
    render(<LinkButton href="/home">Home</LinkButton>);

    expect(screen.getByTestId("LinkButton.a")).toBeInTheDocument();
    expect(screen.getByTestId("LinkButton.button")).toHaveTextContent("Home");
  });
});
