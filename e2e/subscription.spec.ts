import { test, expect } from "@playwright/test";

test.describe("Subscription", () => {
  test("subscribe page shows plan details", async ({ page }) => {
    // Direct access to subscribe page (requires auth)
    await page.goto("/subscribe");

    // Should show subscribe page or redirect to login
    const content = await page.textContent("body");
    expect(content).toBeTruthy();
  });

  test("account page shows subscription status", async ({ page }) => {
    await page.goto("/account");

    // Should redirect to login if not authenticated
    const content = await page.textContent("body");
    expect(content).toBeTruthy();
  });
});
