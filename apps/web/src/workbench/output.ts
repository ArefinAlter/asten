/**
 * The Output panel's channels: what the desk did on the author's behalf
 * (compile, packs, checks, versions, the assistant's tool calls), one line
 * each, kept in memory for the visit. Nothing here leaves the page.
 */
import { useSyncExternalStore } from "react";

export type Channel =
  "Compile" | "Packs" | "Checks" | "Versions" | "Assistant" | "Desk";

export interface OutputLine {
  at: string;
  channel: Channel;
  level: "info" | "warn" | "error";
  text: string;
}

let lines: OutputLine[] = [];
const listeners = new Set<() => void>();

export function log(
  channel: Channel,
  text: string,
  level: OutputLine["level"] = "info",
) {
  lines = [
    ...lines.slice(-499),
    { at: new Date().toISOString(), channel, level, text },
  ];
  listeners.forEach((listener) => listener());
}

export function clearOutput() {
  lines = [];
  listeners.forEach((listener) => listener());
}

export function useOutput(): OutputLine[] {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => lines,
    () => lines,
  );
}
