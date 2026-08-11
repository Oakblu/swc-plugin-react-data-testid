import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig({
  plugins: [
    react({
      plugins: [["swc-plugin-react-data-testid", {}]],
    }),
  ],
  // IterationForms.test.jsx renders the checklist page from example/vite. That file lives
  // outside this package, so React would otherwise be resolved from example/vite's own
  // node_modules and two copies would break hooks.
  resolve: {
    dedupe: ["react", "react-dom"],
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.js"],
  },
});
