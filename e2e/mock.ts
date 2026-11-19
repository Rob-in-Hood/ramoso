import type { Page } from "@playwright/test";

// Canned Rust responses by command name; must run before page.goto.
export async function installMocks(page: Page, responses: Record<string, unknown> = {}) {
  await page.addInitScript((data) => {
    window.__RAMOSO_MOCKS__ = data;
  }, responses);
}

// Sends an event to the app as if Rust emitted it.
export async function emitEvent(page: Page, event: string, payload: unknown) {
  await page.evaluate(([e, p]) => window.__RAMOSO_EMIT__!(e as string, p), [event, payload] as const);
}
