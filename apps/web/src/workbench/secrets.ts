/**
 * Provider keys, held in this page's memory only: never written to storage,
 * never put in a preference, an event, a log or a request in this release.
 * The interface shows "stored" and the last four characters, nothing more.
 */
import { useSyncExternalStore } from "react";

const keys = new Map<string, string>();
const listeners = new Set<() => void>();
let version = 0;

export function storeKey(provider: string, value: string) {
  const trimmed = value.trim();
  if (trimmed) keys.set(provider, trimmed);
  else keys.delete(provider);
  version += 1;
  listeners.forEach((listener) => listener());
}

export function forgetKey(provider: string) {
  storeKey(provider, "");
}

/** What the interface may show about a key: whether it exists and its last four characters. */
export function keyHint(provider: string): { stored: boolean; last4: string } {
  const value = keys.get(provider);
  return { stored: Boolean(value), last4: value ? value.slice(-4) : "" };
}

export function useKeyVersion(): number {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => version,
    () => version,
  );
}
