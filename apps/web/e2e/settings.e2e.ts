import { expect, test } from "@playwright/test";
import { Desk, signIn } from "./desk";

test("Settings shows this desk with a version", async ({ page, request }) => {
  const project = await new Desk(request).createProject();
  await signIn(page);
  await page.goto(`/#/p/${project.id}/settings`);

  await expect(
    page.getByRole("heading", { name: "How the scales are set" }),
  ).toBeVisible();
  const thisDesk = page.getByRole("group", { name: "This desk" });
  await expect(thisDesk).toBeVisible();
  await expect(thisDesk.getByText(/^Asten \d+\.\d+\.\d+/)).toBeVisible();
  await expect(thisDesk).toContainText("The API is a server you started");
  await expect(thisDesk).toContainText("pairing");
});
