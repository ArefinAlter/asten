import { describe, expect, it } from "vitest";
import { INITIAL_LAYOUT } from "./layout";
import { panelSpace } from "./panel-space";

describe("available workspace panel bounds", () => {
  it("keeps an editor visible at every desktop width and short height", () => {
    const requested = {
      ...INITIAL_LAYOUT,
      sidebarWidth: 520,
      secondaryWidth: 640,
      panelHeight: 720,
    };
    for (const width of [960, 1024, 1280, 1366, 1920]) {
      for (const height of [120, 200, 360, 720]) {
        const fitted = panelSpace(requested, width, height, true, true);
        expect(
          width - 48 - fitted.sidebarWidth - fitted.secondaryWidth,
        ).toBeGreaterThanOrEqual(320);
        expect(height - fitted.panelHeight).toBeGreaterThanOrEqual(
          Math.min(160, height / 2),
        );
        expect(fitted.sidebarWidth).toBeGreaterThanOrEqual(200);
        expect(fitted.secondaryWidth).toBeGreaterThanOrEqual(280);
      }
    }
    expect(requested.sidebarWidth).toBe(520);
    expect(requested.secondaryWidth).toBe(640);
  });
  it("restores preferred widths when space returns and ignores hidden peers", () => {
    const requested = {
      ...INITIAL_LAYOUT,
      sidebarWidth: 520,
      secondaryWidth: 640,
    };
    expect(
      panelSpace(requested, 1024, 500, true, true).sidebarWidth,
    ).toBeLessThan(520);
    expect(panelSpace(requested, 1920, 500, true, true).sidebarWidth).toBe(520);
    expect(panelSpace(requested, 1024, 500, true, false).sidebarWidth).toBe(
      520,
    );
  });
  it("does not turn corrupted size values into NaN styles", () => {
    const fitted = panelSpace(
      {
        ...INITIAL_LAYOUT,
        sidebarWidth: NaN,
        secondaryWidth: Infinity,
        panelHeight: NaN,
      },
      1280,
      500,
      true,
      true,
    );
    expect(Object.values(fitted).every(Number.isFinite)).toBe(true);
  });
});
