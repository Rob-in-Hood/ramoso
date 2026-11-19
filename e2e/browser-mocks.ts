// Runs inside the page in e2e builds only: Rust is replaced by canned responses.
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

declare global {
  interface Window {
    __RAMOSO_MOCKS__?: Record<string, unknown>;
    __RAMOSO_EMIT__?: (event: string, payload: unknown) => Promise<void>;
  }
}

// Responses by command name, set by the test before the app loads.
const responses = window.__RAMOSO_MOCKS__ ?? {};

// A command without a response fails loudly instead of returning undefined.
mockWindows("main");
mockIPC(
  (cmd) => {
    if (cmd in responses) return responses[cmd];
    if (cmd.startsWith("plugin:window|")) return null;
    throw new Error(`No mock for command "${cmd}"`);
  },
  { shouldMockEvents: true },
);

// Lets tests send events the way Rust would.
window.__RAMOSO_EMIT__ = (event, payload) => emit(event, payload);
