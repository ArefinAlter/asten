import { type RefObject, useEffect, useRef, useState } from "react";

/*
 * Accessibility helpers for the desk: focus traps for the drawer and the
 * sheet, focus return, and one polite live region that the whole desk shares.
 */

const FOCUSABLE = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
  '[contenteditable="true"]',
].join(",");

/** Elements inside `root` that take focus from the keyboard, in tab order. */
export function focusableIn(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (element) =>
      !element.closest("[inert]") &&
      !element.hasAttribute("hidden") &&
      element.getClientRects().length > 0,
  );
}

function tryFocus(element: HTMLElement | null | undefined): boolean {
  if (!element || !element.isConnected) return false;
  element.focus({ preventScroll: true });
  return document.activeElement === element;
}

/**
 * Remember where focus was when `active` became true and put it back when
 * `active` goes false again. If the opener is gone or hidden by then, focus
 * the `fallback` instead (the toggle that owns the panel).
 */
export function useReturnFocus(
  active: boolean,
  fallback?: RefObject<HTMLElement | null>,
) {
  const opener = useRef<HTMLElement | null>(null);
  const wasActive = useRef(false);
  useEffect(() => {
    if (active) {
      if (!wasActive.current) {
        const current = document.activeElement;
        opener.current =
          current instanceof HTMLElement && current !== document.body
            ? current
            : null;
      }
      wasActive.current = true;
      return;
    }
    if (!wasActive.current) return;
    wasActive.current = false;
    const target = opener.current;
    const second = fallback?.current ?? null;
    opener.current = null;
    // Let the panel's own cleanup (inert, hidden) land first.
    const frame = window.requestAnimationFrame(() => {
      if (!tryFocus(target)) tryFocus(second);
    });
    return () => window.cancelAnimationFrame(frame);
  }, [active, fallback]);
}

interface TrapOptions {
  /** Called on Escape. */
  onClose?: () => void;
  /** Where focus goes when the panel closes if the opener is gone. */
  returnTo?: RefObject<HTMLElement | null>;
  /** Siblings that stay live while the panel is open (the scrim, the live region). */
  keep?: string;
}

/**
 * Hold focus inside a drawer or a sheet while it is open. Everything else in
 * the same parent is marked `inert`, Tab cycles within the panel, Escape asks
 * to close, and focus returns to the opener on close. Focus moves into the
 * panel on open unless something inside already has it (an autofocused field).
 */
export function useFocusTrap(
  ref: RefObject<HTMLElement | null>,
  active: boolean,
  { onClose, returnTo, keep }: TrapOptions = {},
) {
  useReturnFocus(active, returnTo);
  const onCloseRef = useRef(onClose);
  useEffect(() => {
    onCloseRef.current = onClose;
  });

  useEffect(() => {
    const panel = ref.current;
    if (!active || !panel) return;

    const parent = panel.parentElement;
    const made: HTMLElement[] = [];
    if (parent) {
      for (const sibling of Array.from(parent.children)) {
        if (!(sibling instanceof HTMLElement) || sibling === panel) continue;
        if (keep && sibling.matches(keep)) continue;
        if (sibling.inert) continue;
        sibling.inert = true;
        made.push(sibling);
      }
    }

    if (!panel.contains(document.activeElement)) {
      if (!panel.hasAttribute("tabindex")) panel.tabIndex = -1;
      panel.focus({ preventScroll: true });
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.stopPropagation();
        onCloseRef.current?.();
        return;
      }
      if (event.key !== "Tab" || !panel) return;
      const items = focusableIn(panel);
      if (items.length === 0) {
        event.preventDefault();
        return;
      }
      const first = items[0];
      const last = items[items.length - 1];
      const current = document.activeElement;
      if (event.shiftKey && (current === first || current === panel)) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && current === last) {
        event.preventDefault();
        first.focus();
      }
    }
    panel.addEventListener("keydown", onKeyDown);

    return () => {
      panel.removeEventListener("keydown", onKeyDown);
      for (const element of made) element.inert = false;
    };
  }, [ref, active, keep]);
}

/* ───────── One polite live region ───────── */

type Listener = (message: string) => void;
const listeners = new Set<Listener>();
let lastMessage = "";
let lastAt = 0;

/**
 * Say something through the desk's live region. `dedupeMs` drops a repeat
 * of the same message inside that window, so a run of autosaves does not
 * read "Saved" every second.
 */
export function announce(message: string, options: { dedupeMs?: number } = {}) {
  const now = Date.now();
  if (
    options.dedupeMs &&
    message === lastMessage &&
    now - lastAt < options.dedupeMs
  )
    return;
  lastMessage = message;
  lastAt = now;
  for (const listener of listeners) listener(message);
}

/** The current announcement; render it inside one `role="status"` element. */
export function useAnnouncement(): string {
  const [message, setMessage] = useState("");
  useEffect(() => {
    let timer: number | undefined;
    const listener = (next: string) => {
      // Empty the region first so the same words are spoken again.
      setMessage("");
      window.clearTimeout(timer);
      timer = window.setTimeout(() => setMessage(next), 60);
    };
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
      window.clearTimeout(timer);
    };
  }, []);
  return message;
}

/**
 * Announce how many items appeared since the last list for the same `scope`
 * ("3 new marks"). The first list for a scope is a baseline, not news.
 */
export function useAnnounceNew<T extends { id: string }>(
  items: readonly T[] | undefined,
  scope: string | undefined,
  phrase: (count: number) => string,
) {
  const seen = useRef<{ scope: string | undefined; ids: Set<string> } | null>(
    null,
  );
  useEffect(() => {
    if (!items || !scope) return;
    const ids = new Set(items.map((item) => item.id));
    const previous = seen.current;
    seen.current = { scope, ids };
    if (!previous || previous.scope !== scope) return;
    let fresh = 0;
    for (const id of ids) if (!previous.ids.has(id)) fresh += 1;
    if (fresh > 0) announce(phrase(fresh));
  }, [items, scope, phrase]);
}
