import { expect, test } from "@playwright/test";
import { CONTRADICTION, Desk, seedMira, signIn } from "./desk";

const MARK = "CANON-E013";

test("typing a sentence that contradicts a settled fact raises CANON-E013", async ({
  page,
  request,
}) => {
  const { project, chapter19 } = await seedMira(new Desk(request), {
    contradict: false,
  });
  await signIn(page);
  await page.goto(`/#/p/${project.id}/write/${chapter19.id}`);

  const scales = page.getByRole("region", { name: "The Scales" });
  await expect(scales).toContainText("Nothing disagrees with your canon.");
  // The change stream is what carries the new marks back to the desk.
  await expect(page.getByText("live", { exact: true })).toBeVisible();

  const editor = page.locator('[contenteditable="true"]');
  await editor.click();
  await page.keyboard.type(CONTRADICTION);

  const row = scales.getByRole("button", { name: new RegExp(MARK) });
  await expect(row).toBeVisible({ timeout: 30_000 });
  await expect(row).toContainText("auburn");
  // The same mark sits in the page margin beside the sentence.
  await expect(
    page
      .getByRole("complementary", { name: "Marks in the margin" })
      .getByText(MARK, { exact: true }),
  ).toBeVisible();
  // Opening the row shows the evidence on both sides.
  await row.click();
  await expect(scales).toContainText("black");
  await expect(
    scales.getByRole("button", { name: "Add an event", exact: true }),
  ).toBeVisible();
});

test("applying a repair clears the mark", async ({ page, request }) => {
  const desk = new Desk(request);
  const { project, chapter19 } = await seedMira(desk, { contradict: true });
  await expect
    .poll(async () =>
      (await desk.diagnostics(project.id)).some((mark) => mark.code === MARK),
    )
    .toBe(true);

  await signIn(page);
  await page.goto(`/#/p/${project.id}/write/${chapter19.id}`);

  const scales = page.getByRole("region", { name: "The Scales" });
  const row = scales.getByRole("button", { name: new RegExp(MARK) });
  await expect(row).toBeVisible();
  await row.click();
  await scales
    .getByRole("button", { name: "Add an event", exact: true })
    .click();

  await expect(scales.getByText(MARK, { exact: true })).toHaveCount(0, {
    timeout: 30_000,
  });
  await expect(scales).toContainText("Nothing disagrees with your canon.");
  await expect
    .poll(async () =>
      (await desk.diagnostics(project.id)).some((mark) => mark.code === MARK),
    )
    .toBe(false);
});
