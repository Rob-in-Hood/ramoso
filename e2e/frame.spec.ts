import { expect, test } from "@playwright/test";
import { installMocks } from "./mock";

test("app loads with mocked Rust", async ({ page }) => {
  await installMocks(page);
  await page.goto("/");
  await expect(page.getByRole("application", { name: "Ramoso" })).toBeVisible();
});
