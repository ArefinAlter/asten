import { expect, test } from "@playwright/test";
import { signIn, unique } from "./desk";

test("creates a project and a scene from the binder", async ({ page }) => {
  await signIn(page);
  await page.goto("/");

  const projectName = unique("Harbour");
  await page.getByRole("button", { name: "New", exact: true }).click();
  await page.getByLabel("Project name").fill(projectName);
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: projectName, level: 1 }),
  ).toBeVisible();

  const binder = page.getByRole("navigation", { name: "Binder" });
  await expect(binder).toContainText("Empty. Add a scene below.");
  await page.getByRole("button", { name: "+ Add to the binder" }).click();
  await page.getByPlaceholder("Scene title").fill("The first tide");
  await page.getByRole("button", { name: "Add", exact: true }).click();

  await expect(
    binder.getByRole("button", { name: /The first tide/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "The first tide", level: 1 }),
  ).toBeVisible();
  await expect(page.locator('[contenteditable="true"]')).toBeVisible();
});
