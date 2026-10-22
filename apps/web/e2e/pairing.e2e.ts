import { expect, test } from "@playwright/test";
import { Desk } from "./desk";

test("the pair screen accepts a code and reaches the desk", async ({
  page,
  request,
}) => {
  const code = await new Desk(request).mintCode("e2e browser", "desk");

  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Pair this browser with your desk" }),
  ).toBeVisible();

  await page.getByLabel("Pairing code").fill(code);
  await page.getByRole("button", { name: "Pair", exact: true }).click();

  await expect(
    page.getByRole("heading", { name: "A writing desk with a memory." }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Projects" }),
  ).toBeVisible();
});

test("the pair screen rejects a wrong code", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Pairing code").fill("000000");
  await page.getByRole("button", { name: "Pair", exact: true }).click();

  await expect(page.getByRole("alert")).toContainText("not valid");
  await expect(
    page.getByRole("heading", { name: "Pair this browser with your desk" }),
  ).toBeVisible();
});
