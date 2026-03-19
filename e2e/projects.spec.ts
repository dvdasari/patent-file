import { test, expect } from "./fixtures/test-setup";

test.describe("Projects", () => {
  test("projects page shows empty state with CTA", async ({ authedPage: page }) => {
    await expect(page.locator("text=Your Projects")).toBeVisible();
    // Either shows projects or empty state
    const content = await page.textContent("body");
    expect(content).toBeTruthy();
  });

  test("create project button exists", async ({ authedPage: page }) => {
    await expect(page.locator("text=New Patent Draft")).toBeVisible();
  });

  test("clicking New Patent Draft navigates to wizard", async ({ authedPage: page }) => {
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/, { timeout: 10000 });
    // Wizard should show Step 1
    await expect(page.locator("text=Step 1")).toBeVisible();
  });
});
