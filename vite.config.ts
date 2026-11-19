import path from "node:path";
import process from "node:process";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Set by `tauri dev` when the app runs on another device.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],

  // `@/` points at src/.
  resolve: {
    alias: { "@": path.resolve(import.meta.dirname, "./src") },
  },

  // Unit tests live next to the code; Playwright owns e2e/.
  test: {
    include: ["src/**/*.test.{ts,tsx}"],
    passWithNoTests: true,
  },

  // Keep Rust errors visible; Tauri expects this fixed port.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**", "**/spike/**", "**/design/**"] },
  },
});
