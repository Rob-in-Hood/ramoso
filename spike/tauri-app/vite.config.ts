import { defineConfig } from "vite";
// @ts-expect-error no @types/node in this project
import process from "node:process";

// Set by `tauri dev` when developing on another device.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(() => ({
  // Keep Rust errors visible in the terminal.
  clearScreen: false,

  // Tauri expects this exact port.
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,

    // Rust sources are rebuilt by Tauri, not Vite.
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
