import { createContext, useContext } from "react";
import type { Editor } from "@tiptap/react";
import type {
  CanonEntity,
  DiagnosticsSnapshot,
  DocumentSummary,
  Location,
  Project,
  ProjectId,
} from "@asten/contracts";
import type { AstenClient } from "../lib/asten-client";
import type { LayoutAction, LayoutState } from "./layout";

export type WorkbenchView =
  | "overview"
  | "write"
  | "cast"
  | "canon"
  | "time"
  | "world"
  | "history"
  | "guests"
  | "settings"
  | "compile"
  | "packs"
  | "graph"
  | "note"
  | "diff";

export interface AskOptions {
  title: string;
  label: string;
  placeholder?: string;
  initial?: string;
  confirm?: string;
  multiline?: boolean;
}

/** What every part of the workbench may read and do, from one place. */
export interface WorkbenchApi {
  client: AstenClient;
  sample: boolean;
  projectId?: ProjectId;
  project?: Project;
  documents: DocumentSummary[];
  entities: CanonEntity[];
  diagnostics?: DiagnosticsSnapshot;
  openDocumentId?: string;
  selectedMarkId?: string;
  selectedNoteId?: string;
  view: WorkbenchView;
  layout: LayoutState;
  dispatch: (action: LayoutAction) => void;
  editor: Editor | null;
  run: (command: string, arg?: unknown) => void;
  openDocument: (id: string) => void;
  jumpTo: (location: Location) => void;
  selectMark: (id: string | undefined) => void;
  openNote: (id: string) => void;
  setView: (view: WorkbenchView) => void;
  openSettings: (section?: string) => void;
  openDiff: (documentId: string, snapshotId?: string) => void;
  diffTarget?: { documentId: string; snapshotId?: string };
  toast: (text: string, level?: "info" | "warn" | "error") => void;
  ask: (options: AskOptions) => Promise<string | null>;
}

export const WorkbenchContext = createContext<WorkbenchApi | null>(null);

export function useWorkbench(): WorkbenchApi {
  const api = useContext(WorkbenchContext);
  if (!api) throw new Error("useWorkbench outside the workbench");
  return api;
}
