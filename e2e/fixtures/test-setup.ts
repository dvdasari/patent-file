import { test as base, expect } from "@playwright/test";

const API_URL = "http://localhost:5012";

// Test user credentials (created by `make seed-user`)
const TEST_USER = {
  email: "test@example.com",
  password: "testpass123",
};

// Fixture that provides an authenticated page
export const test = base.extend<{ authedPage: ReturnType<typeof base["page"]> }>({
  authedPage: async ({ page }, use) => {
    // Login
    await page.goto("/login");
    await page.fill('input[id="email"]', TEST_USER.email);
    await page.fill('input[id="password"]', TEST_USER.password);
    await page.click('button[type="submit"]');

    // Wait for redirect to projects
    await page.waitForURL("/projects", { timeout: 10000 });

    await use(page);
  },
});

export { expect };
