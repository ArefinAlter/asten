// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useRef } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { usePanelFocus } from "./hooks";

function Panel({ open }: { open: boolean }) {
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = usePanelFocus(open, trigger);
  return (
    <>
      <button ref={trigger}>Open panel</button>
      <aside ref={panel} tabIndex={-1} inert={!open} aria-label="Test panel">
        <input aria-label="First control" />
        <button disabled>Disabled control</button>
        <button>Last control</button>
      </aside>
    </>
  );
}

beforeEach(() => {
  // jsdom has no layout. The browser smoke check verifies real inert/visibility.
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockReturnValue([
    {} as DOMRect,
  ] as unknown as DOMRectList);
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("overlay keyboard focus", () => {
  it("focuses the opened panel and restores its trigger after dismissal", () => {
    const { rerender } = render(<Panel open={false} />);
    const trigger = screen.getByRole("button", { name: "Open panel" });
    trigger.focus();
    rerender(<Panel open />);
    expect(document.activeElement).toBe(screen.getByLabelText("Test panel"));
    screen.getByLabelText("First control").focus();
    rerender(<Panel open={false} />);
    expect(document.activeElement).toBe(trigger);
    expect(screen.getByLabelText("Test panel").hasAttribute("inert")).toBe(
      true,
    );
  });

  it("cycles Tab and Shift-Tab between enabled panel controls", () => {
    render(<Panel open />);
    const panel = screen.getByLabelText("Test panel");
    const first = screen.getByLabelText("First control");
    const last = screen.getByRole("button", { name: "Last control" });
    fireEvent.keyDown(panel, { key: "Tab" });
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(first, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
    fireEvent.keyDown(last, { key: "Tab" });
    expect(document.activeElement).toBe(first);
  });

  it("does not restore focus to a toggle hidden by a desktop resize", () => {
    const { rerender } = render(<Panel open />);
    const trigger = screen.getByRole("button", { name: "Open panel" });
    vi.spyOn(trigger, "getClientRects").mockReturnValue(
      [] as unknown as DOMRectList,
    );
    const control = screen.getByLabelText("First control");
    control.focus();
    rerender(<Panel open={false} />);
    expect(document.activeElement).not.toBe(trigger);
  });
});
