import { test, expect } from "@playwright/test";

test.describe("Authentication", () => {
  test("login page renders email and password fields", async ({ page }) => {
    await page.goto("/login");
    await expect(page.locator('input[id="email"]')).toBeVisible();
    await expect(page.locator('input[id="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
    await expect(page.locator("h1")).toContainText("Patent Draft Pro");
  });

  test("login with invalid credentials shows error", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[id="email"]', "wrong@example.com");
    await page.fill('input[id="password"]', "wrongpass");
    await page.click('button[type="submit"]');

    // Should show error message (stays on login page)
    await expect(page.locator("text=Invalid")).toBeVisible({ timeout: 5000 });
  });

  test("unauthenticated access to /projects redirects to /login", async ({ page }) => {
    await page.goto("/projects");
    // AuthGuard should redirect
    await page.waitForURL(/\/login/, { timeout: 10000 });
  });
});
