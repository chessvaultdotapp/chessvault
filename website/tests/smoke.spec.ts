import { test as base, expect } from "@playwright/test";

// Install listeners before navigation, and check errors even when an assertion fails.
const test = base.extend<{ browserErrors: void }>({
  browserErrors: [
    async ({ page }, use) => {
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(message.text());
      });
      await use();
      expect(errors, "Unexpected browser errors").toEqual([]);
    },
    { auto: true },
  ],
});

for (const [path, heading] of [
  ["", "Chessvault"],
  ["docs/", "What is Chessvault?"],
  ["download/", "Download"],
]) {
  test(`${path || "home"} loads`, async ({ page }) => {
    const response = await page.goto(path || "./");
    expect(response?.status()).toBe(200);
    await expect(page.getByRole("heading", { level: 1, name: heading, exact: true })).toBeVisible();
    await expect(page.getByRole("contentinfo")).toContainText("Chessvault");
    // Catch horizontal overflow that would hide navigation on narrow screens.
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  });
}

test("header navigation reaches documentation, download, and home", async ({ page }) => {
  await page.goto("./");
  const header = page.getByRole("navigation", { name: "Main navigation" });
  for (const [label, path, heading] of [
    ["Documentation", "docs/", "What is Chessvault?"],
    ["Download", "download/", "Download"],
    ["Chessvault", "", "Chessvault"],
  ]) {
    await header.getByRole("link", { name: label, exact: true }).click();
    await expect(page).toHaveURL(`http://127.0.0.1:4321/chessvault/${path}`);
    await expect(page.getByRole("heading", { level: 1, name: heading, exact: true })).toBeVisible();
  }
});

test("theme cycles and persists across navigation and reload", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("./");
  const root = page.locator("html");
  await expect(root).toHaveAttribute("data-theme-preference", "system");
  await expect(root).toHaveAttribute("data-theme", "light");

  await page.getByRole("button", { name: "Theme: system. Switch to light theme", exact: true }).click();
  await expect(root).toHaveAttribute("data-theme-preference", "light");
  await expect(root).toHaveAttribute("data-theme", "light");
  await page.getByRole("button", { name: "Theme: light. Switch to dark theme", exact: true }).click();
  await expect(root).toHaveAttribute("data-theme-preference", "dark");
  await expect(root).toHaveAttribute("data-theme", "dark");

  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("link", { name: "Documentation", exact: true })
    .click();
  await expect(page).toHaveURL(/\/chessvault\/docs\/$/);
  await expect(root).toHaveAttribute("data-theme-preference", "dark");
  await expect(root).toHaveAttribute("data-theme", "dark");
  await page.reload();
  await expect(root).toHaveAttribute("data-theme-preference", "dark");
  await expect(root).toHaveAttribute("data-theme", "dark");

  await page.getByRole("button", { name: "Theme: dark. Switch to system theme", exact: true }).click();
  await expect(root).toHaveAttribute("data-theme-preference", "system");
  await expect(root).toHaveAttribute("data-theme", "light");
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(root).toHaveAttribute("data-theme", "dark");
});

test("external links point to the intended destinations", async ({ page }) => {
  await page.goto("./");
  await expect(page.getByRole("link", { name: "GitHub", exact: true })).toHaveAttribute(
    "href",
    "https://github.com/chessvaultdotapp",
  );

  for (const [path, label] of [
    ["download/", "Browse Linux development previews"],
    ["docs/", "GitHub releases page"],
    ["docs/getting-started/", "GitHub releases page"],
  ]) {
    await page.goto(path);
    await expect(page.getByRole("link", { name: label })).toHaveAttribute(
      "href",
      "https://github.com/chessvaultdotapp/chessvault/releases",
    );
  }
});

test("header links are reachable and usable with the keyboard", async ({ page }) => {
  await page.goto("./");
  const header = page.getByRole("navigation", { name: "Main navigation" });
  for (const [label, path] of [
    ["Documentation", "docs/"],
    ["Download", "download/"],
    ["Chessvault", ""],
  ]) {
    // Each full navigation starts a fresh document's tab order at the brand link.
    const tabs = label === "Documentation" ? 2 : label === "Download" ? 3 : 1;
    for (let i = 0; i < tabs; i++) await page.keyboard.press("Tab");
    const link = header.getByRole("link", { name: label, exact: true });
    await expect(link).toBeFocused();
    await expect(link).toBeInViewport();
    await page.keyboard.press("Enter");
    await expect(page).toHaveURL(`http://127.0.0.1:4321/chessvault/${path}`);
  }
});

test("documentation navigation reaches getting started", async ({ page }) => {
  await page.goto("docs/");
  await page
    .getByRole("navigation", { name: "Documentation", exact: true })
    .getByRole("link", { name: "Getting Started", exact: true })
    .click();
  await expect(page).toHaveURL(/\/chessvault\/docs\/getting-started\/$/);
  await expect(page.getByRole("heading", { level: 1, name: "Getting Started", exact: true })).toBeVisible();
});
