import { expect, test } from "@playwright/test";
import { Desk, signIn } from "./desk";

test("the skip link focuses the workspace without replacing the project URL", async ({
  page,
  request,
}) => {
  const project = await new Desk(request).createProject();
  await signIn(page);
  await page.goto(`/#/p/${project.id}`);
  await expect(
    page.getByRole("heading", { name: project.name, level: 1 }),
  ).toBeVisible();
  const before = page.url();
  await page.getByRole("link", { name: "Skip to the page" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("#workspace")).toBeFocused();
  expect(page.url()).toBe(before);
});

test("a phone drawer traps focus and leaves closed panels inert", async ({
  page,
  request,
}) => {
  const project = await new Desk(request).createProject();
  await signIn(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/#/p/${project.id}`);
  await expect(
    page.getByRole("heading", { name: project.name, level: 1 }),
  ).toBeVisible();
  const sidebar = page.locator("aside.sidebar");
  await expect(sidebar).toHaveAttribute("inert", "");
  const opener = page.getByRole("button", { name: "Open the binder" });
  await opener.click();
  await expect(sidebar).toHaveAttribute("aria-modal", "true");
  await page.keyboard.press("Shift+Tab");
  expect(
    await sidebar.evaluate((node) => node.contains(document.activeElement)),
  ).toBe(true);
  await page.keyboard.press("Tab");
  expect(
    await sidebar.evaluate((node) => node.contains(document.activeElement)),
  ).toBe(true);
  await page.keyboard.press("Escape");
  await expect(opener).toBeFocused();
  await expect(sidebar).toHaveAttribute("inert", "");
});
