import { expect, test } from "@playwright/test";
import { Desk, signIn } from "./desk";

// A phone: one column, the binder as a drawer, the Scales as a bottom sheet.
test.use({ viewport: { width: 390, height: 844 } });

test("on a narrow screen the binder is a drawer and the Scales fold", async ({
  page,
  request,
}) => {
  const desk = new Desk(request);
  const project = await desk.createProject();
  await desk.createDocument(project.id, { title: "Low water" });

  await signIn(page);
  await page.goto(`/#/p/${project.id}`);
  await expect(
    page.getByRole("heading", { name: project.name, level: 1 }),
  ).toBeVisible();

  const binder = page.getByRole("navigation", { name: "Binder" });
  const open = page.getByRole("button", { name: "Open the binder" });
  await expect(open).toBeVisible();
  await expect(binder).not.toBeInViewport();

  await open.click();
  await expect(
    page.getByRole("button", { name: "Close the binder" }),
  ).toBeVisible();
  await expect(binder).toBeInViewport();

  await binder.getByRole("button", { name: /Low water/ }).click();
  await expect(
    page.getByRole("heading", { name: "Low water", level: 1 }),
  ).toBeVisible();
  await expect(open).toBeVisible();
  await expect(binder).not.toBeInViewport();

  const scales = page.getByRole("region", { name: "The Scales" });
  const toggle = scales.getByRole("button", { name: /^The Scales/ });
  const filters = scales.getByRole("tablist", { name: "Filter marks" });
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(filters).toBeVisible();
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(filters).toHaveCount(0);
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(filters).toBeVisible();
});
