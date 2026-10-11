import adapter from "@sveltejs/adapter-static";
import { svelteTesting } from "@testing-library/svelte/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";

const runtime = /** @type {unknown} */ (globalThis);
const { process: nodeProcess } =
  /** @type {{ process: { argv: string[]; env: Record<string, string | undefined> } }} */ (
    runtime
  );

export default defineConfig({
  plugins: [
    sveltekit({
      adapter: adapter({ fallback: "404.html" }),
      paths: {
        base: nodeProcess.argv.includes("dev")
          ? ""
          : /** @type {`/${string}` | undefined} */ (nodeProcess.env.BASE_PATH),
      },
    }),
  ],
  test: {
    projects: [
      {
        extends: true,
        plugins: [svelteTesting()],
        test: {
          name: "client",
          environment: "jsdom",
          clearMocks: true,
          include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
          exclude: ["src/lib/server/**"],
          setupFiles: ["./vitest-setup-client.js"],
        },
      },
      {
        extends: true,
        test: {
          name: "server",
          environment: "node",
          include: ["src/**/*.{test,spec}.{js,ts}"],
          exclude: ["src/**/*.svelte.{test,spec}.{js,ts}"],
        },
      },
    ],
  },
});
