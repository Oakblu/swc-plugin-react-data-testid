import React from "react";
import { render, screen } from "@testing-library/react";
import { UserCard, ProductCard } from "../components/UserCard";

describe("UserCard — data-testid generation", () => {
  it("injects testids on all JSX elements", () => {
    render(<UserCard name="Jane Doe" email="jane@example.com" avatar="👩‍💻" />);

    expect(document.querySelector('[data-testid="UserCard.div"]')).toBeInTheDocument();
    expect(document.querySelector('[data-testid="UserCard.div2"]')).toBeInTheDocument();
    expect(document.querySelector('[data-testid="UserCard.div3"]')).toBeInTheDocument();
    expect(document.querySelector('[data-testid="UserCard.div4"]')).toBeInTheDocument();
    expect(document.querySelector('[data-testid="UserCard.div5"]')).toBeInTheDocument();

    expect(screen.getByTestId("UserCard.h3")).toHaveTextContent("Jane Doe");
    expect(screen.getByTestId("UserCard.p")).toHaveTextContent("jane@example.com");

    expect(screen.getByTestId("UserCard.button")).toHaveTextContent("Follow");
    expect(screen.getByTestId("UserCard.button2")).toHaveTextContent("Message");
  });

  it("never overrides an existing data-testid", () => {
    render(
      <div>
        <UserCard name="Bob" email="bob@example.com" avatar="🧑" />
      </div>
    );

    const cards = document.querySelectorAll('[data-testid^="UserCard."]');
    const ids = Array.from(cards).map((el) => el.getAttribute("data-testid"));
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe("ProductCard — data-testid generation", () => {
  it("scopes counters independently from UserCard", () => {
    render(
      <ProductCard
        title="Widget Pro"
        price="$49.99"
        description="The best widget."
      />
    );

    expect(screen.getByTestId("ProductCard.div")).toBeInTheDocument();
    expect(screen.getByTestId("ProductCard.div2")).toBeInTheDocument();
    expect(screen.getByTestId("ProductCard.h3")).toHaveTextContent("Widget Pro");
    expect(screen.getByTestId("ProductCard.p")).toHaveTextContent("The best widget.");
    expect(screen.getByTestId("ProductCard.span")).toHaveTextContent("$49.99");
    expect(screen.getByTestId("ProductCard.button")).toHaveTextContent("Add to Cart");
  });
});
