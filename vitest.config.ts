import { defineConfig } from "vitest/config";
import { resolve } from "path";

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["src/test.ts"],
    include: ["src/**/*.spec.ts"],
    alias: {
      "@": resolve(__dirname, "src/app"),
    },
  },
});
