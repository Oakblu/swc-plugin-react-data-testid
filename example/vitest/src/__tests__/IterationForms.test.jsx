import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import React from "react";

// The self-checking page from example/vite, rendered headlessly. The component asserts
// its own expectations against the DOM and reports them in a banner, so this test simply
// requires that banner to be green — one automated gate over every iteration form the
// page demonstrates.
import IterationForms from "../../../vite/src/components/IterationForms.jsx";

describe("example/vite iteration-forms checklist", () => {
  it("reports every check green when rendered", () => {
    const { container } = render(<IterationForms />);
    const banner = container.textContent;

    // Surface the failing rows in the assertion message rather than a bare boolean.
    const failures = Array.from(container.querySelectorAll("section"))
      .map((s) => s.textContent)
      .filter((t) => t.startsWith("❌"));

    expect(failures).toEqual([]);
    expect(banner).not.toContain("failed");
    // Pinned to the current count so a check that silently disappears is caught too —
    // "0/0 passed" would otherwise still be green.
    expect(banner).toContain("15/15 checks passed");
  });
});
