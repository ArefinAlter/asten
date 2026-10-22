// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { focusableIn } from "./a11y";

afterEach(() => vi.restoreAllMocks());

it("keeps disabled and roving negative-tabindex controls out of the tab cycle", () => {
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockReturnValue([
    {},
  ] as unknown as DOMRectList);
  const root = document.createElement("div");
  root.innerHTML =
    '<button id="first">First</button><button tabindex="-1">Roving</button><fieldset disabled><input /></fieldset><div inert><button>Hidden</button></div><button id="last">Last</button>';
  expect(focusableIn(root).map((element) => element.id)).toEqual([
    "first",
    "last",
  ]);
});
