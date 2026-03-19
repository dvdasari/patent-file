import { test, expect } from "./fixtures/test-setup";

test.describe("Interview Wizard", () => {
  test("wizard shows step 1 with title and patent type fields", async ({ authedPage: page }) => {
    // Create a project first
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    await expect(page.locator("text=Basics")).toBeVisible();
    await expect(page.locator("text=Invention Title")).toBeVisible();
    await expect(page.locator("text=Patent Type")).toBeVisible();
    await expect(page.locator("text=Technical Field")).toBeVisible();
  });

  test("can navigate between wizard steps", async ({ authedPage: page }) => {
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    // Fill step 1 basics
    await page.fill('input[placeholder*="Irrigation"]', "Test Invention Title");
    await page.selectOption("select", "Software");

    // Go to step 2
    await page.click("text=Next");
    await expect(page.locator("text=Applicant")).toBeVisible();
    await expect(page.locator("text=Step 2")).toBeVisible();

    // Go back to step 1
    await page.click("text=Back");
    await expect(page.locator("text=Step 1")).toBeVisible();
  });

  test("step 7 review shows generate button", async ({ authedPage: page }) => {
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    // Navigate through all steps quickly
    for (let i = 1; i < 7; i++) {
      await page.click("text=Next");
      await page.waitForTimeout(500);
    }

    // Should be on step 7
    await expect(page.locator("text=Review & Generate")).toBeVisible();
    await expect(page.locator("text=Generate Patent Draft")).toBeVisible();
  });
});
