import { test, expect } from "./fixtures/test-setup";

test.describe("Export", () => {
  test("export page shows PDF and DOCX buttons", async ({ authedPage: page }) => {
    // Create project and navigate to export
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    const url = page.url();
    const idMatch = url.match(/id=([^&]+)/);
    if (!idMatch) throw new Error("No project ID");

    await page.goto(`/projects/${idMatch[1]}/export`);

    await expect(page.locator("text=Export Patent Draft")).toBeVisible();
    await expect(page.locator("text=PDF")).toBeVisible();
    await expect(page.locator("text=DOCX")).toBeVisible();
  });
});
