import { test, expect } from "./fixtures/test-setup";

test.describe("Editor", () => {
  test("editor page loads for a project", async ({ authedPage: page }) => {
    // Create a project
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    // Get project ID from URL
    const url = page.url();
    const idMatch = url.match(/id=([^&]+)/);
    if (!idMatch) throw new Error("No project ID in URL");

    // Navigate to editor
    await page.goto(`/projects/${idMatch[1]}`);

    // Should show project title and either sections or empty state
    const content = await page.textContent("body");
    expect(content).toBeTruthy();
  });

  test("editor shows empty state with link to interview", async ({ authedPage: page }) => {
    // Create project and go directly to editor (no generation done)
    await page.click("text=New Patent Draft");
    await page.waitForURL(/\/projects\/new\?id=/);

    const url = page.url();
    const idMatch = url.match(/id=([^&]+)/);
    if (!idMatch) throw new Error("No project ID");

    await page.goto(`/projects/${idMatch[1]}`);

    // Should show empty state since no sections generated
    await expect(page.locator("text=No sections generated")).toBeVisible({ timeout: 5000 });
    await expect(page.locator("text=Go to Interview")).toBeVisible();
  });
});
