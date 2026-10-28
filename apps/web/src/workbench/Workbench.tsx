/**
 * The workbench: Asten's desk as a Word-style ribbon over a VS Code-style
 * layout (activity bar, side bar, bottom panel, assistant panel), with an
 * Obsidian-style knowledge view, drawn in T3 Code's quiet shell. The page in
 * the middle keeps Asten's own typography. Every view the desk had before
 * is still here, reached from the activity bar, the ribbon or Ctrl+K.
 */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Editor } from "@tiptap/react";
import {
  Suspense,
  lazy,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
  type FormEvent,
} from "react";
import type {
  CompileArtifact,
  CompileFormat,
  CreateProject,
  DocumentKind,
  DocumentSummary,
  ImportFormat,
  JsonValue,
  Location,
  ProjectId,
  ProjectKind,
} from "@asten/contracts";
import { panelSpace } from "./panel-space";
import { pointerResize } from "./pointer-resize";
import { Binder } from "../binder/Binder";
import { PaletteMark } from "../brand/Palette";
import { ManuscriptEditor, type Jump } from "../editor/ManuscriptEditor";
import { Inspector } from "../inspector/Inspector";
import { useAnnounceNew, useAnnouncement, useFocusTrap } from "../lib/a11y";
import {
  ApiError,
  createAstenClient,
  isLocalTransport,
} from "../lib/asten-client";
import {
  SAVE_DOCUMENT_KEY,
  useMediaQuery,
  useProjectStream,
} from "../lib/hooks";
import { emptyDoc } from "../lib/pm";
import { Overview } from "../overview/Overview";
import { Packs } from "../packs/Packs";
import type { WorldLens } from "../world/useWorld";
import { AgentPanel } from "./agent/AgentPanel";
import { WindowControls, useCustomChrome } from "./chrome";
import {
  WorkbenchContext,
  type AskOptions,
  type WorkbenchApi,
  type WorkbenchView,
} from "./context";
import { Icon } from "./icons";
import { matchBinding, effectiveBindings, DEFAULT_BINDINGS } from "./keys";
import {
  GraphView,
  KnowledgePane,
  LinksPanel,
  NoteView,
} from "./knowledge/Knowledge";
import {
  INITIAL_LAYOUT,
  layoutReducer,
  visiblePanels,
  type SidePane,
} from "./layout";
import { log } from "./output";
import { Palette, type PaletteItem } from "./Palette";
import { ChecksPane, PacksPane, SearchPane } from "./panes";
import { BottomPanel } from "./panel/BottomPanel";
import { applyPrefs, getPrefs, setPrefs, usePrefs, type Prefs } from "./prefs";
import { deskStore, hostedClaim, isHosted } from "../lib/storage";
import { problemRows, stepMark } from "./problems";
import { Backstage } from "./ribbon/Backstage";
import { resolveCommand, runEditorCommand } from "./ribbon/commands";
import { RIBBON_TABS, type BackstagePage } from "./ribbon/model";
import { OptionsDialog, type OptionPage } from "./ribbon/Options";
import { QAT_CHOICES, Ribbon } from "./ribbon/Ribbon";
import { DiffView, SourceControlPane } from "./scm/SourceControl";
import {
  SECTIONS,
  SettingsView,
  type SectionId,
} from "./settings/SettingsView";
import { downloadArtifact, safeHref } from "./util";

const Canon = lazy(() =>
  import("../canon/Canon").then((m) => ({ default: m.Canon })),
);
const Cast = lazy(() =>
  import("../cast/Cast").then((m) => ({ default: m.Cast })),
);
const CompileDock = lazy(() =>
  import("../compile/CompileDock").then((m) => ({ default: m.CompileDock })),
);
const Guests = lazy(() =>
  import("../guests/Guests").then((m) => ({ default: m.Guests })),
);
const History = lazy(() =>
  import("../history/History").then((m) => ({ default: m.History })),
);
const Timeline = lazy(() =>
  import("../timeline/Timeline").then((m) => ({ default: m.Timeline })),
);
const WorldInspector = lazy(() =>
  import("../world/WorldInspector").then((m) => ({
    default: m.WorldInspector,
  })),
);
const WorldWorkspace = lazy(() =>
  import("../world/WorldWorkspace").then((m) => ({
    default: m.WorldWorkspace,
  })),
);

const client = createAstenClient();
const viewFallback = <p className="status">Opening…</p>;
const KEEP_LIVE = ".scrim, .live-region, .wb-toasts";

const VIEW_LABEL: Record<WorkbenchView, string> = {
  overview: "Overview",
  write: "Write",
  cast: "Cast",
  canon: "Canon",
  time: "Time",
  world: "World",
  history: "History",
  guests: "Guests",
  settings: "Settings",
  compile: "Compile",
  packs: "Packs",
  graph: "Graph view",
  note: "Note",
  diff: "Changes",
};

const VIEWS = new Set(Object.keys(VIEW_LABEL));

const ACTIVITY: {
  pane: SidePane;
  label: string;
  icon: string;
  keys: string;
}[] = [
  {
    pane: "manuscript",
    label: "Manuscript",
    icon: "files",
    keys: "Ctrl+Shift+E",
  },
  {
    pane: "knowledge",
    label: "Knowledge",
    icon: "knowledge",
    keys: "Ctrl+Shift+K",
  },
  { pane: "search", label: "Search", icon: "search", keys: "Ctrl+Shift+F" },
  { pane: "scm", label: "Source Control", icon: "scm", keys: "Ctrl+Shift+G" },
  { pane: "checks", label: "Canon and checks", icon: "checks", keys: "" },
  { pane: "packs", label: "Packs", icon: "packs", keys: "" },
];

function readHash(): {
  project?: string;
  view?: WorkbenchView;
  document?: string;
} {
  const match = /^#\/p\/([^/]+)(?:\/([^/]+))?(?:\/([^/]+))?/.exec(
    window.location.hash,
  );
  if (!match) return {};
  const view =
    match[2] && VIEWS.has(match[2]) ? (match[2] as WorkbenchView) : undefined;
  return {
    project: decodeURIComponent(match[1]),
    view,
    document: match[3] ? decodeURIComponent(match[3]) : undefined,
  };
}

/** Where the desk was left, for "When Asten opens: last page" (desktop and local builds). */
const LAST_KEY = "asten-workbench:last:v1";

const newMarksPhrase = (count: number) =>
  `${count} new mark${count === 1 ? "" : "s"}`;

export function Workbench() {
  const sample = isLocalTransport();
  const hosted = isHosted();
  const prefs = usePrefs();
  const initial = useMemo(() => {
    const fromHash = readHash();
    if (fromHash.project) return fromHash;
    if (getPrefs().openOn !== "last") return fromHash;
    try {
      const last = deskStore.getJson<{
        project?: string;
        document?: string;
      }>(LAST_KEY);
      return last?.project
        ? {
            project: last.project,
            document: last.document,
            view: (last.document ? "write" : "overview") as WorkbenchView,
          }
        : fromHash;
    } catch {
      return fromHash;
    }
  }, []);
  const queryClient = useQueryClient();
  const compact = useMediaQuery("(max-width: 1279px)");
  const narrow = useMediaQuery("(max-width: 959px)");
  const customChrome = useCustomChrome();
  const [layout, dispatch] = useReducer(
    layoutReducer,
    INITIAL_LAYOUT,
    (state) =>
      typeof window !== "undefined" &&
      window.matchMedia("(max-width: 959px)").matches
        ? {
            ...state,
            sidebar: false,
            panel: false,
            secondary: false,
            ribbonCollapsed: true,
          }
        : typeof window !== "undefined" &&
            window.matchMedia("(max-width: 1279px)").matches
          ? { ...state, secondary: false }
          : state,
  );
  const visible = visiblePanels(layout);
  const bodyRef = useRef<HTMLDivElement>(null);
  const resizeStop = useRef<() => void>(() => {});
  const [bodySize, setBodySize] = useState({
    width: window.innerWidth,
    height: Math.max(0, window.innerHeight - 200),
  });
  useEffect(() => {
    const body = bodyRef.current;
    if (!body) return;
    const measure = () => {
      const rect = body.getBoundingClientRect();
      setBodySize((previous) =>
        previous.width === rect.width && previous.height === rect.height
          ? previous
          : { width: rect.width, height: rect.height },
      );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(body);
    return () => {
      observer.disconnect();
      resizeStop.current();
    };
  }, []);
  const fitted = panelSpace(
    layout,
    bodySize.width,
    bodySize.height,
    visible.sidebar && !narrow,
    visible.secondary && !narrow,
  );

  const [selectedProjectId, setSelectedProjectId] = useState<
    ProjectId | undefined
  >(initial.project);
  const [selectedDocumentId, setSelectedDocumentId] = useState<
    string | undefined
  >(initial.document);
  const [view, setViewState] = useState<WorkbenchView>(
    initial.view ?? "overview",
  );
  const [worldLens, setWorldLens] = useState<WorldLens>("wall");
  const [selectedWorldId, setSelectedWorldId] = useState<string>();
  const [selectedEntityKey, setSelectedEntityKey] = useState<string>();
  const [selectedMarkId, setSelectedMarkId] = useState<string>();
  const [selectedNoteId, setSelectedNoteId] = useState<string>();
  const [diffTarget, setDiffTarget] = useState<{
    documentId: string;
    snapshotId?: string;
  }>();
  const [jump, setJump] = useState<Jump>();
  const [isCreating, setIsCreating] = useState(false);
  const [trackChanges, setTrackChanges] = useState(false);
  const [showBlame, setShowBlame] = useState(false);
  const [findOpen, setFindOpen] = useState(false);
  const [findQuery, setFindQuery] = useState("");
  const [artifact, setArtifact] = useState<CompileArtifact>();
  const [saveError, setSaveError] = useState<string>();
  const [editor, setEditor] = useState<Editor | null>(null);
  const [editorStatus, setEditorStatus] = useState({
    words: 0,
    characters: 0,
    save: "saved",
  });
  const [palette, setPalette] = useState<{
    items: PaletteItem[];
    placeholder: string;
    query?: string;
  } | null>(null);
  const [backstage, setBackstage] = useState<BackstagePage | null>(null);
  const [options, setOptions] = useState<OptionPage | null>(null);
  const [settingsSection, setSettingsSection] = useState<SectionId>("general");
  const [splitDocId, setSplitDocId] = useState<string>();
  const [toasts, setToasts] = useState<
    { id: number; text: string; level: string }[]
  >([]);
  const [asking, setAsking] = useState<
    (AskOptions & { resolve: (value: string | null) => void }) | null
  >(null);
  const [drawer, setDrawer] = useState(false);
  const connected = useProjectStream(selectedProjectId);

  useEffect(() => applyPrefs(getPrefs()), []);

  const setView = useCallback((next: WorkbenchView) => {
    setViewState(next);
    if (next !== "write") setFindOpen(false);
  }, []);

  // The address follows the desk, and the desk follows the address.
  useEffect(() => {
    const parts = selectedProjectId
      ? [`#/p/${selectedProjectId}`, view, selectedDocumentId ?? ""]
      : ["#/"];
    const next = parts.filter(Boolean).join("/");
    if (window.location.hash !== next)
      window.history.replaceState(null, "", next);
  }, [selectedProjectId, view, selectedDocumentId]);
  useEffect(() => {
    if (!selectedProjectId) return;
    deskStore.setJson(LAST_KEY, {
      project: selectedProjectId,
      document: selectedDocumentId,
    });
  }, [selectedProjectId, selectedDocumentId]);
  useEffect(() => {
    const onHashChange = () => {
      const target = readHash();
      if (!target.project) return;
      setSelectedProjectId(target.project as ProjectId);
      if (target.view) setViewState(target.view);
      setSelectedDocumentId(target.document);
    };
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => client.listProjects(),
  });
  // A project deleted elsewhere (or just now) is no longer the open one.
  useEffect(() => {
    if (
      selectedProjectId &&
      projects.data &&
      !projects.data.some((p) => p.id === selectedProjectId)
    ) {
      setSelectedProjectId(undefined);
      setSelectedDocumentId(undefined);
      setViewState("overview");
    }
  }, [projects.data, selectedProjectId]);
  // The hosted copies open their one project straight away.
  useEffect(() => {
    if ((sample || hosted) && !selectedProjectId && projects.data?.length === 1)
      setSelectedProjectId(projects.data[0].id);
  }, [projects.data, selectedProjectId, sample, hosted]);
  const documents = useQuery({
    queryKey: ["documents", selectedProjectId],
    queryFn: () => client.listDocumentSummaries(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const openDocument = useQuery({
    queryKey: ["document", selectedProjectId, selectedDocumentId],
    queryFn: () => client.getDocument(selectedProjectId!, selectedDocumentId!),
    enabled: Boolean(selectedProjectId && selectedDocumentId),
  });
  const entities = useQuery({
    queryKey: ["entities", selectedProjectId],
    queryFn: () => client.listEntities(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const diagnostics = useQuery({
    queryKey: ["diagnostics", selectedProjectId],
    queryFn: () => client.diagnostics(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const blame = useQuery({
    queryKey: ["blame", selectedProjectId, selectedDocumentId],
    queryFn: () => client.blame(selectedProjectId!, selectedDocumentId!),
    enabled: Boolean(selectedProjectId && selectedDocumentId && showBlame),
  });
  const splitDocument = useQuery({
    queryKey: ["document", selectedProjectId, splitDocId],
    queryFn: () => client.getDocument(selectedProjectId!, splitDocId!),
    enabled: Boolean(selectedProjectId && splitDocId && layout.split),
  });

  const announcement = useAnnouncement();
  useAnnounceNew(
    diagnostics.data?.diagnostics,
    selectedProjectId,
    newMarksPhrase,
  );

  const sidebarRef = useRef<HTMLElement>(null);
  const drawerToggleRef = useRef<HTMLButtonElement>(null);
  useFocusTrap(sidebarRef, narrow && drawer, {
    onClose: () => setDrawer(false),
    returnTo: drawerToggleRef,
    keep: KEEP_LIVE,
  });
  useEffect(() => {
    if (!narrow) setDrawer(false);
  }, [narrow]);
  useEffect(() => {
    if (narrow) setDrawer(false);
  }, [narrow, view, selectedDocumentId]);

  const toast = useCallback(
    (text: string, level: "info" | "warn" | "error" = "info") => {
      const id = Date.now() + Math.random();
      setToasts((all) => [...all.slice(-3), { id, text, level }]);
      window.setTimeout(
        () => setToasts((all) => all.filter((t) => t.id !== id)),
        4200,
      );
    },
    [],
  );
  const ask = useCallback(
    (opts: AskOptions) =>
      new Promise<string | null>((resolve) => setAsking({ ...opts, resolve })),
    [],
  );

  const createProject = useMutation({
    mutationFn: (input: CreateProject) => client.createProject(input),
    onSuccess: async (project) => {
      setSelectedProjectId(project.id);
      setIsCreating(false);
      setView("overview");
      await queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
  const createDocument = useMutation({
    mutationFn: (input: {
      title: string;
      kind: DocumentKind;
      parent_id?: string | null;
    }) =>
      client.createDocument({
        project_id: selectedProjectId!,
        kind: input.kind,
        title: input.title,
        parent_id: input.parent_id,
        content_json: emptyDoc(),
        position: (documents.data?.length ?? 0) + 1,
      }),
    onSuccess: async (document) => {
      setSelectedDocumentId(document.id);
      setView("write");
      await queryClient.invalidateQueries({ queryKey: ["documents"] });
      await queryClient.invalidateQueries({ queryKey: ["documents-full"] });
    },
  });
  const updateDocument = useMutation({
    mutationKey: SAVE_DOCUMENT_KEY,
    mutationFn: (input: {
      documentId: string;
      content_json?: JsonValue;
      exclude_from_compile?: boolean;
      title?: string;
      position?: number;
      parent_id?: string | null;
      expected_head_seq?: number;
    }) =>
      client.updateDocument(selectedProjectId!, input.documentId, {
        content_json: input.content_json,
        exclude_from_compile: input.exclude_from_compile,
        title: input.title,
        position: input.position,
        parent_id: input.parent_id,
        expected_head_seq: input.expected_head_seq,
        actor: "author",
      }),
    onSuccess: async (saved) => {
      setSaveError(undefined);
      queryClient.setQueryData(["document", saved.project_id, saved.id], saved);
      await queryClient.invalidateQueries({
        queryKey: ["documents", selectedProjectId],
      });
    },
    onError: (error, input) => {
      setSaveError(error.message);
      if (error instanceof ApiError && error.status === 409)
        void queryClient.invalidateQueries({
          queryKey: ["document", selectedProjectId, input.documentId],
        });
    },
  });
  const deleteDocument = useMutation({
    mutationFn: (documentId: string) =>
      client.deleteDocument(selectedProjectId!, documentId),
    onSuccess: async (_, documentId) => {
      setSelectedDocumentId(undefined);
      queryClient.removeQueries({
        queryKey: ["document", selectedProjectId, documentId],
      });
      await queryClient.invalidateQueries({
        queryKey: ["documents", selectedProjectId],
      });
    },
  });
  const compile = useMutation({
    mutationFn: (format: CompileFormat) =>
      client.compileProject(selectedProjectId!, { format }),
    onSuccess: (result, format) => {
      setArtifact(result);
      log("Compile", `Compiled ${result.filename} (${format}).`);
      if (getPrefs().snapshotOnCompile && selectedDocumentId)
        void client.createSnapshot(
          selectedProjectId!,
          selectedDocumentId,
          `Compiled to ${format}`,
        );
    },
    onError: (error, format) =>
      log("Compile", `${format}: ${error.message}`, "error"),
  });
  const importDocument = useMutation({
    mutationFn: async (file: File) => {
      if (/\.(rtf|odt|pages|pdf|doc)$/i.test(file.name))
        throw new Error(
          "Asten reads .docx, .md, .txt, .html, .fountain and .fdx. Save the file from your word processor as one of those and import that.",
        );
      const format = importFormat(file.name);
      const bytes = new Uint8Array(await file.arrayBuffer());
      // Text goes as UTF-8 when it is; otherwise (UTF-16, Windows-1252) the
      // bytes go as they are and the engine reads the byte-order mark.
      let body: string;
      let encoding: "utf-8" | "base64" = "base64";
      if (format === "docx") body = toBase64(bytes);
      else
        try {
          body = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
          encoding = "utf-8";
        } catch {
          body = toBase64(bytes);
        }
      return client.importDocument(selectedProjectId!, {
        filename: file.name,
        format,
        body,
        encoding,
        split: "auto",
      });
    },
    onSuccess: async (document) => {
      // A long file arrives as a manuscript with a document per chapter:
      // say so, and open the first chapter rather than the empty container.
      const all = await client.listDocumentSummaries(selectedProjectId!);
      const chapters = all
        .filter((d) => d.parent_id === document.id)
        .sort((a, b) => a.position - b.position);
      log(
        "Desk",
        chapters.length
          ? `Imported “${document.title}” as ${chapters.length} chapters.`
          : `Imported “${document.title}”.`,
      );
      toast(
        chapters.length
          ? `Imported ${chapters.length} chapters under “${document.title}”.`
          : `Imported “${document.title}”.`,
      );
      setSelectedDocumentId(chapters[0]?.id ?? document.id);
      setView("write");
      await queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
    onError: (error) => {
      log("Desk", `Import failed: ${error.message}`, "error");
      toast(error.message, "error");
    },
  });

  const seedSample = useMutation({
    mutationFn: () =>
      import("../sample/seed").then((module) =>
        module.createSampleProject(client),
      ),
    onSuccess: async (projectId) => {
      await queryClient.invalidateQueries({ queryKey: ["projects"] });
      log(
        "Desk",
        "Created the sample novel as a project; the engine weighs it like any other.",
      );
      setBackstage(null);
      selectProject(projectId);
    },
    onError: (error) => toast(error.message, "error"),
  });
  // The hosted pre-alpha starts each new visitor on the sample novel, created
  // through the real API in their own sandbox; the claim makes it once only,
  // even with two tabs open.
  const seededRef = useRef(false);
  useEffect(() => {
    if (!hosted || seededRef.current || projects.data?.length !== 0) return;
    seededRef.current = true;
    void hostedClaim("sample-novel").then((first) => {
      if (first) seedSample.mutate();
    });
  }, [hosted, projects.data, seedSample]);

  const selectedProject = projects.data?.find(
    (p) => p.id === selectedProjectId,
  );
  const summaries = useMemo(() => documents.data ?? [], [documents.data]);
  const selectedSummary = summaries.find((d) => d.id === selectedDocumentId);
  const selectedDocument =
    openDocument.data?.id === selectedDocumentId
      ? openDocument.data
      : undefined;
  const marks = useMemo(
    () => diagnostics.data?.diagnostics ?? [],
    [diagnostics.data],
  );
  const openMarks = marks.filter((m) => !m.suppressed_by);

  const jumpTo = useCallback(
    (location: Location) => {
      setSelectedDocumentId(location.document);
      setView("write");
      setJump({
        block: location.block,
        start: location.start,
        end: location.end,
        nonce: Date.now(),
      });
    },
    [setView],
  );

  function selectProject(id: ProjectId) {
    setSelectedProjectId(id);
    setSelectedDocumentId(undefined);
    setSelectedWorldId(undefined);
    setSelectedEntityKey(undefined);
    setSelectedMarkId(undefined);
    setSelectedNoteId(undefined);
    setView("overview");
  }

  function submitProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createProject.mutate({
      name: String(form.get("name") ?? ""),
      kind: String(form.get("kind") ?? "novel") as ProjectKind,
    });
  }

  const openDocumentById = useCallback(
    (id: string) => {
      setSelectedDocumentId(id);
      setView("write");
    },
    [setView],
  );

  const openNote = useCallback(
    (id: string) => {
      setSelectedNoteId(id);
      setView("note");
    },
    [setView],
  );

  const openSettings = useCallback(
    (section?: string) => {
      if (section && SECTIONS.some((s) => s.id === section))
        setSettingsSection(section as SectionId);
      setView("settings");
    },
    [setView],
  );

  const openDiff = useCallback(
    (documentId: string, snapshotId?: string) => {
      setDiffTarget({ documentId, snapshotId });
      setView("diff");
    },
    [setView],
  );

  // ───────────────────────── commands ─────────────────────────

  const parentForNew = () => {
    if (!selectedSummary)
      return summaries.find((d) => d.kind === "chapter")?.id ?? null;
    if (
      selectedSummary.kind === "chapter" ||
      selectedSummary.kind === "manuscript"
    )
      return selectedSummary.id;
    return selectedSummary.parent_id;
  };

  const pickFrom = (placeholder: string, items: PaletteItem[]) =>
    setPalette({ placeholder, items });

  const insertText = (text: string) =>
    editor?.chain().focus().insertContent(text).run();

  const prefsPreset = (patch: Partial<Prefs>, label: string) => {
    setPrefs(patch);
    toast(`${label} applied to the page display.`);
  };

  const runRef = useRef<(id: string, arg?: unknown) => void>(() => undefined);
  const run = useCallback(
    (id: string, arg?: unknown) => runRef.current(id, arg),
    [],
  );

  runRef.current = (id: string, arg?: unknown) => {
    const resolved = resolveCommand(id);
    if (
      resolved.kind === "unavailable" &&
      !id.startsWith("palette") &&
      !id.startsWith("layout") &&
      !id.startsWith("pane") &&
      !id.startsWith("switcher")
    ) {
      toast(resolved.reason);
      return;
    }
    if (resolved.kind === "editor") {
      if (!editor) return toast("Open a page in the binder first.");
      if (!runEditorCommand(editor, id)) toast("That does not apply here.");
      return;
    }
    const needEditor = () => {
      if (!editor) toast("Open a page in the binder first.");
      return Boolean(editor);
    };
    switch (id) {
      case "palette.open":
        return setPalette({
          placeholder: "Search pages, notes, marks and commands",
          items: allItems(),
        });
      case "palette.commands":
        return setPalette({
          placeholder: "Run a command",
          items: allItems(),
          query: ">",
        });
      case "switcher.open":
        return setPalette({
          placeholder: "Find or open a page or note",
          items: fileItems(),
        });
      case "layout.sidebar":
        return narrow
          ? setDrawer((v) => !v)
          : dispatch({ type: "toggleSidebar" });
      case "layout.panel":
        return dispatch({ type: "togglePanel" });
      case "layout.secondary":
      case "view.secondary":
        return dispatch({ type: "toggleSecondary" });
      case "layout.ribbon":
        return dispatch({ type: "toggleRibbon" });
      case "layout.maximizePanel":
        return dispatch({ type: "toggleMaximizePanel" });
      case "panel.toggle":
        return dispatch({ type: "togglePanel" });
      case "panel.problems":
        return dispatch({ type: "showPanel", tab: "problems" });
      case "panel.terminal":
        return dispatch({ type: "showPanel", tab: "terminal" });
      case "pane.manuscript":
      case "pane.knowledge":
      case "pane.search":
      case "pane.scm":
        if (narrow) setDrawer(true);
        return dispatch({ type: "showPane", pane: id.slice(5) as SidePane });
      case "app.settings":
        return openSettings();
      case "app.backstage":
        return setBackstage("home");
      case "app.paste":
        if (!needEditor()) return;
        void navigator.clipboard
          ?.readText()
          .then((text) => text && insertText(text))
          .catch(() =>
            toast(
              "Use Ctrl+V: the browser does not let the ribbon read the clipboard.",
            ),
          );
        return;
      case "app.cut":
      case "app.copy":
        if (!needEditor()) return;
        editor!.commands.focus();
        document.execCommand(id === "app.cut" ? "cut" : "copy");
        return;
      case "app.find":
        if (view !== "write")
          return toast(
            "Find works in the open page; Search (Ctrl+Shift+F) looks across the project.",
          );
        setFindOpen(true);
        window.setTimeout(
          () =>
            document.querySelector<HTMLInputElement>(".find-input")?.focus(),
          0,
        );
        return;
      case "app.newScene":
      case "app.newChapter":
      case "app.newNote": {
        if (!selectedProjectId) return toast("Open a project first.");
        const kind: DocumentKind =
          id === "app.newScene"
            ? "scene"
            : id === "app.newChapter"
              ? "chapter"
              : "note";
        void ask({
          title: `New ${kind}`,
          label: "Title",
          placeholder:
            kind === "note" ? "Research: harbour charts" : "Untitled",
        }).then((title) => {
          if (!title) return;
          const notesFolder =
            summaries.find((d) => d.kind === "note" && !d.parent_id)?.id ??
            null;
          createDocument.mutate({
            title,
            kind,
            parent_id:
              kind === "scene"
                ? parentForNew()
                : kind === "chapter"
                  ? (summaries.find((d) => d.kind === "manuscript")?.id ?? null)
                  : notesFolder,
          });
        });
        return;
      }
      case "app.comment": {
        if (!needEditor() || !selectedDocumentId) return;
        const { from, to } = editor!.state.selection;
        const $from = editor!.state.doc.resolve(from);
        const block = $from.node(1);
        const blockStart = $from.start(1);
        void ask({
          title: "New comment",
          label: "Comment",
          multiline: true,
          placeholder: "A note to yourself; never part of the manuscript",
        }).then(async (body) => {
          if (!body) return;
          await client.createComment(selectedProjectId!, selectedDocumentId, {
            block_id: (block?.attrs?.id as string | undefined) ?? null,
            from_offset: Math.max(0, from - blockStart),
            to_offset: Math.max(0, to - blockStart),
            body,
          });
          await queryClient.invalidateQueries({ queryKey: ["comments"] });
          dispatch({ type: "showSecondary", tab: "inspector" });
          if (!layout.secondary || layout.secondaryTab !== "inspector")
            dispatch({ type: "showSecondary", tab: "inspector" });
          toast("Comment added. It is in the Inspector.");
        });
        return;
      }
      case "app.showComments":
        return dispatch({ type: "showSecondary", tab: "inspector" });
      case "app.link":
        if (!needEditor()) return;
        void ask({
          title: "Insert link",
          label: "Address",
          placeholder: "https://example.org/",
        }).then((href) => {
          if (!href) return;
          const safe = safeHref(href);
          if (!safe)
            return toast("Links must start with https:// or mailto:.", "warn");
          const empty = editor!.state.selection.empty;
          if (empty)
            editor!
              .chain()
              .focus()
              .insertContent({
                type: "text",
                text: safe,
                marks: [{ type: "link", attrs: { href: safe } }],
              })
              .run();
          else
            editor!
              .chain()
              .focus()
              .extendMarkRange("link")
              .setLink({ href: safe })
              .run();
        });
        return;
      case "app.crossReference":
        if (!needEditor()) return;
        return pickFrom(
          "Cross-reference: pick a page",
          summaries
            .filter((d) => d.id !== selectedDocumentId)
            .map((d) => ({
              id: d.id,
              label: d.title,
              group: d.kind,
              icon: "crossRef",
              run: () => {
                const href = `#/p/${selectedProjectId}/write/${d.id}`;
                if (editor!.state.selection.empty)
                  editor!
                    .chain()
                    .focus()
                    .insertContent({
                      type: "text",
                      text: d.title,
                      marks: [{ type: "link", attrs: { href } }],
                    })
                    .run();
                else editor!.chain().focus().setLink({ href }).run();
              },
            })),
        );
      case "app.insertCharacter":
      case "app.insertPlace":
        if (!needEditor()) return;
        return pickFrom(
          id === "app.insertCharacter"
            ? "Insert a character's name"
            : "Insert a place name",
          (entities.data ?? [])
            .filter(
              (e) =>
                e.kind ===
                (id === "app.insertCharacter" ? "character" : "place"),
            )
            .flatMap((e) =>
              [e.name, ...e.aliases].map((name) => ({
                id: `${e.id}:${name}`,
                label: name,
                group: e.name,
                icon: id === "app.insertCharacter" ? "person" : "pin",
                run: () => insertText(name),
              })),
            ),
        );
      case "app.citation":
        if (!needEditor()) return;
        return pickFrom(
          "Cite a source note",
          summaries
            .filter(
              (d) =>
                d.kind === "source" || d.kind === "note" || d.kind === "wiki",
            )
            .map((d) => ({
              id: d.id,
              label: d.title,
              group: d.kind,
              icon: "source",
              run: () => {
                const href = `#/p/${selectedProjectId}/write/${d.id}`;
                if (editor!.state.selection.empty)
                  toast("Select the words the source supports, then cite it.");
                else editor!.chain().focus().setLink({ href }).run();
              },
            })),
        );
      case "app.preset.standard":
        return prefsPreset(
          {
            pageFont: "mono",
            pageFontSize: "16",
            lineSpacing: "2",
            pageMargins: "wide",
            pageSize: "letter",
            pageColumns: "1",
          },
          "Standard manuscript format",
        );
      case "app.preset.book":
        return prefsPreset(
          {
            pageFont: "literata",
            pageFontSize: "17",
            lineSpacing: "1.5",
            pageMargins: "normal",
            pageSize: "trade",
            pageColumns: "1",
          },
          "Book format",
        );
      case "app.preset.screenplay":
        return prefsPreset(
          {
            pageFont: "mono",
            pageFontSize: "16",
            lineSpacing: "1.5",
            pageMargins: "normal",
            pageSize: "letter",
            pageColumns: "1",
          },
          "Screenplay format",
        );
      case "app.master.chapter":
      case "app.master.scene":
      case "app.master.front":
        setPrefs({ master: id.slice(11) as Prefs["master"] });
        return;
      case "app.scheme.papyrus":
      case "app.scheme.lapis":
      case "app.scheme.malachite":
      case "app.scheme.ink":
        setPrefs({ scheme: id.slice(11) as Prefs["scheme"] });
        return;
      case "app.openCanon":
      case "app.proposals":
        return setView("canon");
      case "app.openCast":
        if (typeof arg === "string") setSelectedEntityKey(arg);
        return setView("cast");
      case "app.openTimeline":
        return setView("time");
      case "app.sources":
        dispatch({ type: "showPane", pane: "knowledge" });
        return;
      case "app.factCard":
        dispatch({ type: "showSecondary", tab: "inspector" });
        return toast(
          "Hover a name on the page for its fact card; the Inspector keeps it.",
        );
      case "app.runChecks":
        if (!selectedProjectId) return toast("Open a project first.");
        log("Checks", `Weighing ${selectedProject?.name ?? "the project"}…`);
        void queryClient
          .invalidateQueries({ queryKey: ["diagnostics", selectedProjectId] })
          .then(async () => {
            const snap = await client.diagnostics(selectedProjectId);
            const open = snap.diagnostics.filter((d) => !d.suppressed_by);
            log(
              "Checks",
              `${open.length} open mark${open.length === 1 ? "" : "s"} at revision ${snap.revision.slice(0, 8)}.`,
            );
          });
        dispatch({ type: "showPanel", tab: "problems" });
        return;
      case "app.nextMark":
      case "app.prevMark": {
        const rows = problemRows(marks, summaries);
        const next = stepMark(
          rows,
          selectedMarkId,
          id === "app.nextMark" ? 1 : -1,
        );
        if (!next) return toast("No open marks.");
        setSelectedMarkId(next.id);
        if (next.diagnostic.primary) jumpTo(next.diagnostic.primary);
        return;
      }
      case "app.trackChanges":
        return setTrackChanges((v) => !v);
      case "app.blame":
        return setShowBlame((v) => !v);
      case "app.compare":
        if (selectedDocumentId) openDiff(selectedDocumentId);
        dispatch({ type: "showPane", pane: "scm" });
        return;
      case "app.save":
        if (!selectedDocumentId || !selectedProjectId)
          return toast("Open a page to save a version of it.");
        void ask({
          title: "Save a version",
          label: "Message",
          placeholder: "What changed",
        }).then(async (label) => {
          if (!label) return;
          await client.createSnapshot(
            selectedProjectId,
            selectedDocumentId,
            label,
          );
          await queryClient.invalidateQueries({ queryKey: ["snapshots"] });
          log("Versions", `Committed “${selectedSummary?.title}”: ${label}`);
          toast("Version saved. It is in Source Control.");
        });
        return;
      case "app.compileQuick":
        return setBackstage("export");
      case "app.print":
        if (view !== "write") return toast("Open a page to print it.");
        window.print();
        return;
      case "app.theme": {
        const order: Prefs["theme"][] = ["light", "dark", "sepia", "system"];
        const next =
          order[(order.indexOf(getPrefs().theme) + 1) % order.length];
        setPrefs({ theme: next });
        return toast(`Theme: ${next}`);
      }
      case "app.shortcuts":
        return openSettings("keybindings");
      case "app.about":
        return openSettings("about");
      case "app.rules":
        return openSettings("assistant");
      case "view.mode.read":
      case "view.mode.print":
      case "view.mode.draft":
      case "view.mode.focus": {
        const mode = id.slice(10) as typeof layout.pageMode;
        dispatch({
          type: "pageMode",
          mode: layout.pageMode === mode && mode === "focus" ? "print" : mode,
        });
        if (view !== "write" && selectedDocumentId) setView("write");
        return;
      }
      case "view.ruler":
        return dispatch({ type: "toggle", key: "ruler" });
      case "view.navigationPane":
        return dispatch({ type: "toggle", key: "navigationPane" });
      case "view.split":
        if (!layout.split && !splitDocId)
          setSplitDocId(
            summaries.find((d) => d.kind === "wiki" || d.kind === "note")?.id,
          );
        return dispatch({ type: "toggle", key: "split" });
      case "view.zoomIn":
        return dispatch({ type: "zoomBy", delta: 10 });
      case "view.zoomOut":
        return dispatch({ type: "zoomBy", delta: -10 });
      case "view.zoom100":
        return dispatch({ type: "zoom", zoom: 100 });
      case "view.graph":
        return setView("graph");
      default:
        if (id.startsWith("view.go."))
          return setView(id.slice(8) as WorkbenchView);
        if (id.startsWith("settings.")) return openSettings(id.slice(9));
        toast("Not in this release.");
    }
  };

  // ───────────────────────── palette items ─────────────────────────

  const bindings = effectiveBindings(prefs.keybindings);
  const keyOf = (command: string) =>
    bindings.find((b) => b.command === command)?.keys;

  function fileItems(): PaletteItem[] {
    return [
      ...summaries.map((d) => ({
        id: d.id,
        label: d.title,
        group: "Pages",
        detail: d.kind,
        icon:
          d.kind === "scene"
            ? "file"
            : d.kind === "chapter" || d.kind === "manuscript"
              ? "folder"
              : "note",
        run: () =>
          d.kind === "wiki" || d.kind === "note" || d.kind === "source"
            ? openNote(`doc:${d.id}`)
            : openDocumentById(d.id),
      })),
      ...(entities.data ?? []).map((e) => ({
        id: e.id,
        label: e.name,
        group: "Notes",
        detail: e.kind,
        icon:
          e.kind === "character"
            ? "person"
            : e.kind === "place"
              ? "pin"
              : "key",
        run: () => openNote(`entity:${e.id}`),
      })),
    ];
  }

  function allItems(): PaletteItem[] {
    const commands: PaletteItem[] = [
      ...DEFAULT_BINDINGS.map((b) => ({
        id: b.command,
        label: b.title,
        group: "Commands",
        keys: keyOf(b.command),
        icon: "chevronRight",
        run: () => run(b.command),
      })),
      ...(
        [
          "overview",
          "write",
          "cast",
          "canon",
          "time",
          "world",
          "history",
          "compile",
          "packs",
          "graph",
        ] as WorkbenchView[]
      ).map((v) => ({
        id: `view.go.${v}`,
        label: `Go to ${VIEW_LABEL[v]}`,
        group: "Commands",
        icon: "chevronRight",
        disabled:
          !selectedProjectId && v !== "packs"
            ? "Open a project first."
            : undefined,
        run: () => setView(v),
      })),
      ...SECTIONS.map((s) => ({
        id: `settings.${s.id}`,
        label: `Settings: ${s.label}`,
        group: "Commands",
        icon: s.icon,
        run: () => openSettings(s.id),
      })),
      ...RIBBON_TABS.flatMap((tab) =>
        tab.groups.flatMap((g) =>
          g.controls
            .filter((c) => !c.select)
            .map((c) => {
              const resolved = resolveCommand(c.id);
              return {
                id: `${tab.id}:${c.id}`,
                label: `${tab.label}: ${c.label}`,
                group: "Ribbon",
                icon: c.icon,
                keys: c.keys,
                disabled:
                  resolved.kind === "unavailable" ? resolved.reason : undefined,
                run: () => run(c.id),
              };
            }),
        ),
      ),
      {
        id: "options",
        label: "File: Options",
        group: "Commands",
        icon: "settings",
        run: () => setOptions("General"),
      },
      ...(
        [
          "home",
          "new",
          "open",
          "info",
          "export",
          "print",
          "share",
          "account",
        ] as BackstagePage[]
      ).map((p) => ({
        id: `bs.${p}`,
        label: `File: ${p[0].toUpperCase()}${p.slice(1)}`,
        group: "Commands",
        icon: "folder",
        run: () => setBackstage(p),
      })),
    ];
    const markItems: PaletteItem[] = openMarks.map((m) => ({
      id: m.id,
      label: `${m.code} ${m.title}`,
      group: "Marks",
      detail: summaries.find((d) => d.id === m.primary?.document)?.title,
      icon: "problems",
      run: () => {
        setSelectedMarkId(m.id);
        if (m.primary) jumpTo(m.primary);
      },
    }));
    const projectItems: PaletteItem[] = (projects.data ?? []).map((p) => ({
      id: p.id,
      label: p.name,
      group: "Projects",
      icon: "book",
      run: () => selectProject(p.id),
    }));
    return [...fileItems(), ...markItems, ...projectItems, ...commands];
  }

  // ───────────────────────── keyboard ─────────────────────────

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (palette || backstage || options || asking) return;
      const target = event.target as HTMLElement | null;
      const editing = Boolean(target?.closest?.("[contenteditable='true']"));
      const typing =
        Boolean(target?.closest?.("input, textarea, select")) && !editing;
      if (event.key === "Escape") {
        if (layout.pageMode === "focus")
          dispatch({ type: "pageMode", mode: "print" });
        setDrawer(false);
        return;
      }
      const binding = matchBinding(bindings, event, editing);
      if (!binding) return;
      if (
        typing &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.altKey &&
        !/^F\d+$/.test(event.key)
      )
        return;
      event.preventDefault();
      run(binding.command);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [bindings, palette, backstage, options, asking, layout.pageMode, run]);

  // Automatic versions, if the person asked for them.
  useEffect(() => {
    if (!prefs.autoSnapshotMinutes || !selectedProjectId || !selectedDocumentId)
      return;
    const timer = window.setInterval(() => {
      void client
        .createSnapshot(
          selectedProjectId,
          selectedDocumentId,
          "Automatic version",
        )
        .then(() => {
          log("Versions", "Automatic version saved.");
          void queryClient.invalidateQueries({ queryKey: ["snapshots"] });
        });
    }, prefs.autoSnapshotMinutes * 60_000);
    return () => window.clearInterval(timer);
  }, [
    prefs.autoSnapshotMinutes,
    selectedProjectId,
    selectedDocumentId,
    queryClient,
  ]);

  useEffect(() => {
    document.querySelector(".page-prose")?.setAttribute("lang", prefs.language);
    document
      .querySelector(".page-prose")
      ?.setAttribute("spellcheck", String(prefs.spellcheck));
  }, [prefs.spellcheck, prefs.language, editor]);

  // ───────────────────────── resizing ─────────────────────────

  const resizeBounds = (target: "sidebar" | "secondary" | "panel") => {
    const body = bodyRef.current;
    if (!body) return { min: 0, max: 0 };
    const rect = body.getBoundingClientRect();
    if (target === "panel") {
      const max = Math.max(
        0,
        Math.floor(rect.height - Math.min(160, rect.height / 2)),
      );
      return { min: Math.min(120, max), max };
    }
    const peer = body.querySelector<HTMLElement>(
      target === "sidebar" ? ".secondary" : ".wb-sidebar",
    );
    const peerWidth = peer && !narrow ? peer.getBoundingClientRect().width : 0;
    const max = Math.max(
      0,
      Math.min(
        target === "sidebar" ? 520 : 640,
        rect.width - (narrow ? 0 : 48) - peerWidth - (narrow ? 0 : 320),
      ),
    );
    const min = Math.min(target === "sidebar" ? 200 : 280, max);
    return { min, max };
  };
  const startResize =
    (target: "sidebar" | "secondary" | "panel") =>
    (event: React.PointerEvent) => {
      if (event.button !== 0 || event.isPrimary === false) return;
      resizeStop.current();
      const panel = event.currentTarget.parentElement;
      const rect = panel?.getBoundingClientRect();
      const initial =
        target === "panel"
          ? (rect?.height ?? fitted.panelHeight)
          : (rect?.width ??
            (target === "sidebar"
              ? fitted.sidebarWidth
              : fitted.secondaryWidth));
      // A person editing the layout starts from the actual peer sizes, preventing a stale
      // larger preference from making the other panel jump as the drag begins.
      if (target === "sidebar")
        dispatch({
          type: "resize",
          target: "secondary",
          size: fitted.secondaryWidth,
        });
      if (target === "secondary")
        dispatch({
          type: "resize",
          target: "sidebar",
          size: fitted.sidebarWidth,
        });
      resizeStop.current = pointerResize(
        event,
        event.currentTarget as HTMLElement,
        {
          axis: target === "panel" ? "y" : "x",
          direction: target === "sidebar" ? 1 : -1,
          initial,
          bounds: () => resizeBounds(target),
          change: (size) => dispatch({ type: "resize", target, size }),
        },
      );
    };
  const resizeKey =
    (target: "sidebar" | "secondary" | "panel") =>
    (event: React.KeyboardEvent) => {
      const lower =
        target === "panel"
          ? "ArrowDown"
          : target === "sidebar"
            ? "ArrowLeft"
            : "ArrowRight";
      const higher =
        target === "panel"
          ? "ArrowUp"
          : target === "sidebar"
            ? "ArrowRight"
            : "ArrowLeft";
      if (![lower, higher, "Home", "End"].includes(event.key)) return;
      event.preventDefault();
      event.stopPropagation();
      const { min, max } = resizeBounds(target);
      const current =
        target === "panel"
          ? fitted.panelHeight
          : target === "sidebar"
            ? fitted.sidebarWidth
            : fitted.secondaryWidth;
      const size =
        event.key === "Home"
          ? min
          : event.key === "End"
            ? max
            : Math.max(
                min,
                Math.min(
                  max,
                  current +
                    (event.key === higher ? 1 : -1) *
                      (event.shiftKey ? 50 : 10),
                ),
              );
      dispatch({ type: "resize", target, size });
    };

  const api: WorkbenchApi = {
    client,
    sample,
    projectId: selectedProjectId,
    project: selectedProject,
    documents: summaries,
    entities: entities.data ?? [],
    diagnostics: diagnostics.data,
    openDocumentId: selectedDocumentId,
    selectedMarkId,
    selectedNoteId,
    view,
    layout,
    dispatch,
    editor,
    run,
    openDocument: openDocumentById,
    jumpTo,
    selectMark: setSelectedMarkId,
    openNote,
    setView,
    openSettings,
    openDiff,
    diffTarget,
    toast,
    ask,
  };

  const errorCount = openMarks.filter((m) => m.severity === "error").length;
  const warningCount = openMarks.filter((m) => m.severity === "warning").length;
  const qat = prefs.qat
    .map((qid) => QAT_CHOICES.find((c) => c.id === qid))
    .filter(Boolean) as typeof QAT_CHOICES;
  const pageTitle =
    view === "write" && selectedSummary
      ? selectedSummary.title
      : view === "settings"
        ? "Settings"
        : VIEW_LABEL[view];
  useEffect(() => {
    document.title = selectedProject
      ? `${pageTitle} · ${selectedProject.name} · Asten`
      : "Asten";
  }, [pageTitle, selectedProject]);

  const shellClass = [
    "wb",
    narrow ? "wb--narrow" : "",
    compact ? "wb--compact" : "",
    customChrome ? "chrome-custom" : "",
    `mode-${layout.pageMode}`,
    visible.sidebar && !narrow ? "has-sidebar" : "",
    visible.secondary && !narrow ? "has-secondary" : "",
    drawer ? "drawer-open" : "",
    !prefs.showMarksInline ? "hide-inline-marks" : "",
    !prefs.showMargin ? "hide-margin" : "",
  ]
    .filter(Boolean)
    .join(" ");

  const sidePane = (
    <>
      <div className="side-head">
        <span>{ACTIVITY.find((a) => a.pane === layout.sidePane)?.label}</span>
        {layout.sidePane === "manuscript" && (
          <button
            type="button"
            className="linkish"
            onClick={() => setIsCreating(true)}
          >
            New
          </button>
        )}
      </div>
      <div className="side-body">
        {layout.sidePane === "manuscript" && (
          <>
            {projects.isPending && <p className="pane-empty">Loading…</p>}
            {projects.error && (
              <p className="wb-error">{projects.error.message}</p>
            )}
            <nav className="tree project-tree" aria-label="Projects">
              {projects.data?.map((project) => (
                <button
                  className={project.id === selectedProjectId ? "selected" : ""}
                  key={project.id}
                  onClick={() => selectProject(project.id)}
                  type="button"
                >
                  <Icon name="book" size={14} />
                  <span>{project.name}</span>
                  <small>{project.kind.replace("_", " ")}</small>
                </button>
              ))}
            </nav>
            {isCreating && (
              <form className="new-project" onSubmit={submitProject}>
                <label>
                  Project name
                  <input name="name" autoFocus required />
                </label>
                <label>
                  Type
                  <select name="kind" defaultValue="novel">
                    <option value="novel">Novel</option>
                    <option value="screenplay">Screenplay</option>
                    <option value="non_fiction">Non-fiction</option>
                    <option value="other">Other</option>
                  </select>
                </label>
                <div className="row">
                  <button
                    type="submit"
                    className="wb-btn wb-btn--small wb-btn--primary"
                    disabled={createProject.isPending}
                  >
                    Create
                  </button>
                  <button
                    type="button"
                    className="wb-btn wb-btn--small"
                    onClick={() => setIsCreating(false)}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            )}
            {selectedProject && (
              <>
                <div className="pane-section">Binder</div>
                <Binder
                  projectId={selectedProject.id}
                  documents={summaries}
                  selectedId={selectedDocumentId}
                  marks={openMarks}
                  onSelect={(document: DocumentSummary) =>
                    openDocumentById(document.id)
                  }
                  onCreate={(input) => createDocument.mutate(input)}
                  creating={createDocument.isPending}
                  onToggleCompile={(document) =>
                    updateDocument.mutate({
                      documentId: document.id,
                      exclude_from_compile: !document.exclude_from_compile,
                    })
                  }
                  onRename={(document, title) =>
                    updateDocument.mutate({ documentId: document.id, title })
                  }
                  onMove={(document, position, parent_id) =>
                    updateDocument.mutate({
                      documentId: document.id,
                      position,
                      parent_id,
                    })
                  }
                  onDelete={(document) => {
                    if (
                      !getPrefs().confirmDelete ||
                      window.confirm(
                        `Delete “${document.title}”? Its history goes with it.`,
                      )
                    )
                      deleteDocument.mutate(document.id);
                  }}
                />
              </>
            )}
          </>
        )}
        {layout.sidePane === "knowledge" && <KnowledgePane />}
        {layout.sidePane === "search" && <SearchPane />}
        {layout.sidePane === "scm" && <SourceControlPane />}
        {layout.sidePane === "checks" && <ChecksPane />}
        {layout.sidePane === "packs" && <PacksPane />}
      </div>
    </>
  );

  const writeArea = selectedProject && (
    <section className="write-pane">
      {saveError && (
        <p className="wb-error banner" role="alert">
          {saveError}
        </p>
      )}
      {findOpen && (
        <div className="find-bar">
          <Icon name="search" />
          <input
            className="find-input"
            placeholder="Find in the page"
            value={findQuery}
            onChange={(e) => setFindQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setFindOpen(false);
                setFindQuery("");
              }
            }}
            aria-label="Find in the page"
          />
          <button
            type="button"
            className="wb-icon-btn"
            aria-label="Close find"
            onClick={() => {
              setFindOpen(false);
              setFindQuery("");
            }}
          >
            <Icon name="x" />
          </button>
        </div>
      )}
      <div
        className={`write-layout${layout.navigationPane && !narrow ? " with-nav" : ""}${layout.split && !narrow ? " with-split" : ""}`}
      >
        {layout.navigationPane && !narrow && (
          <NavigationPane documentId={selectedDocumentId} editor={editor} />
        )}
        <div className="page-scroll">
          {layout.ruler && layout.pageMode !== "read" && <Ruler />}
          <div className="page-zoom" style={{ zoom: layout.zoom / 100 }}>
            {selectedDocument ? (
              layout.pageMode === "read" ? (
                <ReadView
                  title={selectedDocument.title}
                  kind={selectedDocument.kind}
                  content={selectedDocument.content_json}
                />
              ) : (
                <ManuscriptEditor
                  key={selectedDocument.id}
                  projectId={selectedProject.id}
                  document={selectedDocument}
                  diagnostics={marks}
                  entities={entities.data ?? []}
                  pageView={layout.pageMode !== "draft"}
                  trackChanges={trackChanges}
                  findQuery={findOpen ? findQuery : ""}
                  showBlame={showBlame}
                  blame={blame.data}
                  selectedMarkId={selectedMarkId}
                  jump={jump}
                  saving={updateDocument.isPending}
                  onSave={(content_json, headSeq) =>
                    updateDocument.mutateAsync({
                      documentId: selectedDocument.id,
                      content_json,
                      expected_head_seq: headSeq,
                    })
                  }
                  onSelectMark={(markId) => {
                    setSelectedMarkId(markId);
                    if (!layout.panel)
                      dispatch({ type: "showPanel", tab: "problems" });
                  }}
                  onSelectEntity={(key) => {
                    setSelectedEntityKey(key);
                    setView("cast");
                  }}
                  onEditor={setEditor}
                  hideToolbar
                  onStatus={setEditorStatus}
                />
              )
            ) : selectedDocumentId && openDocument.isPending ? (
              <p className="status">Opening the page…</p>
            ) : openDocument.error ? (
              <section className="editor-placeholder register">
                <p className="eyebrow mono">Binder</p>
                <h1>That page could not be opened</h1>
                <p className="error">{openDocument.error.message}</p>
              </section>
            ) : (
              <section className="editor-placeholder register">
                <p className="eyebrow mono">Binder</p>
                <h1>Start a scene</h1>
                <p>
                  Add a scene in the binder, or import a manuscript from File,
                  Open.
                </p>
              </section>
            )}
          </div>
        </div>
        {layout.split && !narrow && (
          <aside className="split-pane" aria-label="Second page, read only">
            <div className="split-head">
              <select
                value={splitDocId ?? ""}
                onChange={(e) => setSplitDocId(e.target.value)}
                aria-label="Page in the split"
              >
                <option value="" disabled>
                  Choose a page
                </option>
                {summaries.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.title}
                  </option>
                ))}
              </select>
              <span className="muted small">read only</span>
              <button
                type="button"
                className="wb-icon-btn"
                aria-label="Close split"
                onClick={() => dispatch({ type: "toggle", key: "split" })}
              >
                <Icon name="x" />
              </button>
            </div>
            {splitDocument.data ? (
              <ReadView
                title={splitDocument.data.title}
                kind={splitDocument.data.kind}
                content={splitDocument.data.content_json}
                compact
              />
            ) : (
              <p className="pane-empty">
                Choose a page to read beside this one.
              </p>
            )}
          </aside>
        )}
      </div>
    </section>
  );

  const mainView = !selectedProject ? (
    view === "settings" ? (
      <SettingsView section={settingsSection} onSection={setSettingsSection} />
    ) : view === "guests" ? (
      <Guests />
    ) : view === "packs" ? (
      <Packs />
    ) : (
      <section className="empty-state">
        <PaletteMark className="hero-mark" />
        <h1>A writing desk with a memory.</h1>
        <p>
          {projects.data?.length === 0
            ? "Your words stay black. Asten only ever writes in red."
            : "Select a project or create one. Your words stay black. Asten only ever writes in red."}
        </p>
        <button
          type="button"
          className="wb-btn wb-btn--primary"
          onClick={() => setBackstage(projects.data?.length ? "open" : "new")}
        >
          {projects.data?.length ? "Open a project" : "Create a project"}
        </button>
        {!sample && (
          <button
            type="button"
            className="wb-btn"
            disabled={seedSample.isPending}
            onClick={() => seedSample.mutate()}
          >
            {seedSample.isPending ? "Creating…" : "Open the sample novel"}
          </button>
        )}
      </section>
    )
  ) : view === "world" ? (
    <WorldWorkspace
      projectId={selectedProject.id}
      projectName={selectedProject.name}
      lens={worldLens}
      selectedId={selectedWorldId}
      onLensChange={setWorldLens}
      onSelect={setSelectedWorldId}
      onWriteScene={() => setView("write")}
    />
  ) : view === "cast" ? (
    <Cast
      projectId={selectedProject.id}
      selectedKey={selectedEntityKey}
      onSelectKey={setSelectedEntityKey}
      documents={summaries}
      onJump={jumpTo}
    />
  ) : view === "canon" ? (
    <Canon
      projectId={selectedProject.id}
      documents={summaries}
      entities={entities.data ?? []}
      snapshot={diagnostics.data}
    />
  ) : view === "time" ? (
    <Timeline
      projectId={selectedProject.id}
      documents={summaries}
      entities={entities.data ?? []}
      onOpenDocument={(docId) =>
        jumpTo({ document: docId, start: 0, end: 0, quote: "", label: "" })
      }
    />
  ) : view === "history" ? (
    <History
      projectId={selectedProject.id}
      documents={summaries}
      documentId={selectedDocumentId}
      onSelectDocument={setSelectedDocumentId}
    />
  ) : view === "guests" ? (
    <Guests />
  ) : view === "settings" ? (
    <SettingsView section={settingsSection} onSection={setSettingsSection} />
  ) : view === "compile" ? (
    <CompileDock
      compiling={compile.isPending}
      artifact={artifact}
      onCompile={(format) => compile.mutate(format)}
      onImportFile={(file) => importDocument.mutate(file)}
      error={compile.error?.message ?? importDocument.error?.message}
    />
  ) : view === "packs" ? (
    <Packs />
  ) : view === "graph" ? (
    <GraphView />
  ) : view === "note" ? (
    <NoteView />
  ) : view === "diff" ? (
    <DiffView />
  ) : view === "write" ? (
    writeArea
  ) : (
    <Overview
      project={selectedProject}
      documents={summaries}
      entities={entities.data ?? []}
      snapshot={diagnostics.data}
      onGo={(v) => setView(v as WorkbenchView)}
      onOpenDocument={(d) => openDocumentById(d.id)}
    />
  );

  const secondary = (
    <aside className="secondary" aria-label="Assistant and inspector">
      <div
        className="secondary-resizer"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize the right panel"
        onPointerDown={startResize("secondary")}
        onKeyDown={resizeKey("secondary")}
        tabIndex={0}
        aria-valuenow={Math.min(fitted.secondaryWidth, bodySize.width)}
        aria-valuemin={resizeBounds("secondary").min}
        aria-valuemax={resizeBounds("secondary").max}
      />
      <div className="secondary-tabs" role="tablist">
        {(["agent", "inspector", "links"] as const).map((tab) => (
          <button
            key={tab}
            type="button"
            role="tab"
            aria-selected={layout.secondaryTab === tab}
            className={layout.secondaryTab === tab ? "active" : ""}
            onClick={() => dispatch({ type: "showSecondary", tab })}
          >
            {tab === "agent"
              ? "Assistant"
              : tab === "inspector"
                ? "Inspector"
                : "Links"}
          </button>
        ))}
        <span className="spacer" />
        <button
          type="button"
          className="wb-icon-btn"
          aria-label="Close the right panel"
          title="Close (Ctrl+Alt+B)"
          onClick={() => dispatch({ type: "toggleSecondary" })}
        >
          <Icon name="x" />
        </button>
      </div>
      <div className="secondary-body">
        {layout.secondaryTab === "agent" && <AgentPanel />}
        {layout.secondaryTab === "links" && <LinksPanel />}
        {layout.secondaryTab === "inspector" &&
          (selectedProject && view === "world" ? (
            <Suspense fallback={viewFallback}>
              <WorldInspector
                projectId={selectedProject.id}
                selectedId={selectedWorldId}
              />
            </Suspense>
          ) : selectedProject ? (
            <Inspector
              projectId={selectedProject.id}
              document={selectedSummary}
              documents={summaries}
              entityKey={selectedEntityKey}
              marks={marks}
              selectedMarkId={selectedMarkId}
              onJump={jumpTo}
              onOpenEntity={(key) => {
                setSelectedEntityKey(key);
                setView("cast");
              }}
              onSelectMark={setSelectedMarkId}
            />
          ) : (
            <p className="pane-empty">
              The inspector shows what is settled about the name under your
              cursor, the page's notes, and the mark you selected.
            </p>
          ))}
      </div>
    </aside>
  );

  return (
    <WorkbenchContext.Provider value={api}>
      <div
        className={shellClass}
        style={{
          ["--wb-sidebar-w" as string]: `${fitted.sidebarWidth}px`,
          ["--wb-secondary-w" as string]: `${fitted.secondaryWidth}px`,
          ["--wb-panel-h" as string]: `${fitted.panelHeight}px`,
        }}
      >
        <a
          className="skip-link"
          href="#workspace"
          onClick={(e) => {
            e.preventDefault();
            window.document.getElementById("workspace")?.focus();
          }}
        >
          Skip to the page
        </a>
        <div className="sr-only live-region" role="status" aria-live="polite">
          {announcement}
        </div>

        <header className="wb-title" data-tauri-drag-region>
          <button
            type="button"
            ref={drawerToggleRef}
            className="wb-icon-btn binder-toggle"
            aria-label={drawer ? "Close the binder" : "Open the binder"}
            aria-expanded={drawer}
            onClick={() => setDrawer((v) => !v)}
          >
            <Icon name="sidebar" />
          </button>
          <span className="wb-brand" data-tauri-drag-region>
            <PaletteMark />
            <span className="wb-wordmark" data-tauri-drag-region>
              Asten
            </span>
          </span>
          <div className="qat" role="toolbar" aria-label="Quick Access Toolbar">
            {qat.map((item) => (
              <button
                key={item.id}
                type="button"
                className="wb-icon-btn"
                title={item.label}
                aria-label={item.label}
                onMouseDown={(e) =>
                  item.id.startsWith("editor.") && e.preventDefault()
                }
                onClick={() => run(item.id)}
              >
                <Icon name={item.icon} />
              </button>
            ))}
            <button
              type="button"
              className="wb-icon-btn qat-more"
              title="Customize the Quick Access Toolbar"
              aria-label="Customize the Quick Access Toolbar"
              onClick={() => setOptions("Quick Access Toolbar")}
            >
              <Icon name="chevronDown" size={12} />
            </button>
          </div>
          <div className="wb-title-text" data-tauri-drag-region>
            <span className="crumb" data-tauri-drag-region>
              {selectedProject?.name ?? "Project library"}
            </span>
            {selectedProject && (
              <span className="crumb-sep" data-tauri-drag-region>
                /
              </span>
            )}
            {selectedProject && (
              <span className="crumb-page" data-tauri-drag-region>
                {pageTitle}
              </span>
            )}
          </div>
          <button
            type="button"
            className="wb-search"
            onClick={() => run("palette.open")}
            aria-label="Search and commands"
          >
            <Icon name="search" size={14} />
            <span>Search</span>
            <kbd>Ctrl</kbd>
            <kbd>K</kbd>
          </button>
          <span
            className="wb-pill prealpha"
            title={
              sample
                ? "This release runs on sample data with no backend. See About."
                : "Pre-alpha release"
            }
          >
            Pre-alpha release
          </span>
          <div className="layout-toggles">
            <button
              type="button"
              className={`wb-icon-btn${layout.sidebar ? " active" : ""}`}
              title="Toggle side bar (Ctrl+B)"
              aria-label="Toggle side bar"
              aria-pressed={layout.sidebar}
              onClick={() => run("layout.sidebar")}
            >
              <Icon name="sidebar" />
            </button>
            <button
              type="button"
              className={`wb-icon-btn${layout.panel ? " active" : ""}`}
              title="Toggle panel (Ctrl+J)"
              aria-label="Toggle panel"
              aria-pressed={layout.panel}
              onClick={() => run("layout.panel")}
            >
              <Icon name="panel" />
            </button>
            <button
              type="button"
              className={`wb-icon-btn${layout.secondary ? " active" : ""}`}
              title="Toggle assistant panel (Ctrl+Alt+B)"
              aria-label="Toggle assistant panel"
              aria-pressed={layout.secondary}
              onClick={() => run("layout.secondary")}
            >
              <Icon name="sidebarRight" />
            </button>
          </div>
          {customChrome && <WindowControls />}
        </header>

        {visible.ribbon || layout.ribbonCollapsed
          ? layout.pageMode !== "focus" && (
              <Ribbon
                trackChanges={trackChanges}
                blame={showBlame}
                onFile={() => setBackstage("home")}
              />
            )
          : null}

        <div className="wb-body" ref={bodyRef}>
          <nav className="activitybar" aria-label="Workspace views">
            {ACTIVITY.map((a) => (
              <button
                key={a.pane}
                type="button"
                title={`${a.label}${a.keys ? ` (${a.keys})` : ""}`}
                aria-label={a.label}
                aria-pressed={visible.sidebar && layout.sidePane === a.pane}
                className={
                  (narrow ? drawer : visible.sidebar) &&
                  layout.sidePane === a.pane
                    ? "selected"
                    : ""
                }
                onClick={() => {
                  if (narrow) {
                    dispatch({ type: "showPane", pane: a.pane });
                    if (layout.sidePane === a.pane && drawer) setDrawer(false);
                    else setDrawer(true);
                    if (!layout.sidebar) dispatch({ type: "toggleSidebar" });
                  } else dispatch({ type: "showPane", pane: a.pane });
                }}
              >
                <Icon name={a.icon} size={20} />
                {a.pane === "checks" && openMarks.length > 0 && (
                  <span className="activity-badge">{openMarks.length}</span>
                )}
                <span className="activity-label">{a.label}</span>
              </button>
            ))}
            <span className="activity-spacer" />
            <button
              type="button"
              title="Graph view (Ctrl+G)"
              aria-label="Graph view"
              className={view === "graph" ? "selected" : ""}
              onClick={() => setView("graph")}
              disabled={!selectedProject}
            >
              <Icon name="graph" size={20} />
              <span className="activity-label">Graph</span>
            </button>
            <button
              type="button"
              title="Settings (Ctrl+,)"
              aria-label="Settings"
              className={view === "settings" ? "selected" : ""}
              onClick={() => openSettings()}
            >
              <Icon name="settings" size={20} />
              <span className="activity-label">Settings</span>
            </button>
          </nav>

          {(visible.sidebar || narrow) && (
            <aside
              className="sidebar wb-sidebar"
              ref={sidebarRef}
              inert={narrow && !drawer}
              role={narrow && drawer ? "dialog" : undefined}
              aria-modal={(narrow && drawer) || undefined}
              aria-label="Projects and binder"
              tabIndex={-1}
            >
              {sidePane}
              {!narrow && (
                <div
                  className="sidebar-resizer"
                  role="separator"
                  aria-orientation="vertical"
                  aria-label="Resize the side bar"
                  onPointerDown={startResize("sidebar")}
                  onKeyDown={resizeKey("sidebar")}
                  tabIndex={0}
                  aria-valuenow={fitted.sidebarWidth}
                  aria-valuemin={resizeBounds("sidebar").min}
                  aria-valuemax={resizeBounds("sidebar").max}
                />
              )}
            </aside>
          )}
          {narrow && drawer && (
            <button
              type="button"
              className="scrim"
              aria-label="Close the panel"
              onClick={() => setDrawer(false)}
            />
          )}

          <div
            className={`wb-center${layout.panelMaximized && visible.panel ? " panel-max" : ""}`}
          >
            <main
              id="workspace"
              tabIndex={-1}
              className={`workspace wb-workspace view-${view}`}
            >
              <Suspense fallback={viewFallback}>{mainView}</Suspense>
            </main>
            {visible.panel && (
              <BottomPanel
                onResizeStart={startResize("panel")}
                onResizeKey={resizeKey("panel")}
                panelSize={fitted.panelHeight}
                panelMax={fitted.panelMax}
              />
            )}
          </div>

          {visible.secondary && secondary}
        </div>

        <footer className="statusbar wb-status">
          <button
            type="button"
            className={`status-item live ${connected ? "on" : ""}`}
            title={
              sample
                ? "Sample backend in this page"
                : connected
                  ? "Live: changes stream from the API"
                  : "Not connected to the change stream"
            }
            onClick={() => openSettings("storage")}
          >
            <span className="dot" />{" "}
            {sample ? "sample" : connected ? "live" : "offline"}
          </button>
          <button
            type="button"
            className="status-item"
            onClick={() => run("panel.problems")}
            title="Problems (Ctrl+Shift+M)"
          >
            <span className="sev-dot sev-error" /> {errorCount}{" "}
            <span className="sev-dot sev-warning" /> {warningCount}
          </button>
          {view === "write" && selectedSummary && (
            <>
              <span className="status-item">
                Page {Math.max(1, Math.ceil(editorStatus.words / 300))} of{" "}
                {Math.max(1, Math.ceil(editorStatus.words / 300))}
              </span>
              <span className="status-item">
                {editorStatus.words.toLocaleString("en")} words
              </span>
              <span className="status-item muted">{editorStatus.save}</span>
            </>
          )}
          <span className="spacer" />
          <span className="status-item">
            {prefs.language === "en-GB" ? "English (UK)" : prefs.language}
          </span>
          {diagnostics.data && (
            <span className="status-item mono" title="Revision">
              rev {diagnostics.data.revision.slice(0, 8)}
            </span>
          )}
          <div className="status-views" role="group" aria-label="Page view">
            {(["read", "print", "draft", "focus"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                className={layout.pageMode === mode ? "active" : ""}
                title={`${mode === "print" ? "Print layout" : mode[0].toUpperCase() + mode.slice(1)} view`}
                aria-label={`${mode} view`}
                aria-pressed={layout.pageMode === mode}
                onClick={() => run(`view.mode.${mode}`)}
              >
                <Icon name={mode === "print" ? "page" : mode} size={14} />
              </button>
            ))}
          </div>
          <div className="status-zoom">
            <button
              type="button"
              aria-label="Zoom out"
              onClick={() => run("view.zoomOut")}
            >
              −
            </button>
            <input
              type="range"
              min={50}
              max={200}
              step={10}
              value={layout.zoom}
              onChange={(e) =>
                dispatch({ type: "zoom", zoom: Number(e.target.value) })
              }
              aria-label="Zoom"
            />
            <button
              type="button"
              aria-label="Zoom in"
              onClick={() => run("view.zoomIn")}
            >
              +
            </button>
            <button
              type="button"
              className="zoom-value"
              onClick={() => run("view.zoom100")}
              title="Reset zoom"
            >
              {layout.zoom}%
            </button>
          </div>
        </footer>

        {palette && (
          <Palette
            items={palette.items}
            placeholder={palette.placeholder}
            initialQuery={palette.query}
            onClose={() => setPalette(null)}
          />
        )}
        {backstage && (
          <Backstage
            page={backstage}
            onPage={setBackstage}
            onClose={() => setBackstage(null)}
            onOptions={() => setOptions("General")}
            onImport={(file) => {
              setBackstage(null);
              importDocument.mutate(file);
            }}
            onSelectProject={(pid) => selectProject(pid)}
            onSample={sample ? undefined : () => seedSample.mutate()}
          />
        )}
        {options && (
          <OptionsDialog initial={options} onClose={() => setOptions(null)} />
        )}
        {asking && (
          <AskDialog
            options={asking}
            onDone={(value) => {
              asking.resolve(value);
              setAsking(null);
            }}
          />
        )}
        <div className="wb-toasts" aria-live="polite">
          {toasts.map((t) => (
            <div key={t.id} className={`wb-toast toast-${t.level}`}>
              <span className="dot" />
              {t.text}
            </div>
          ))}
        </div>
        {artifact && view !== "compile" && (
          <ArtifactNotice
            artifact={artifact}
            onClose={() => setArtifact(undefined)}
          />
        )}
      </div>
    </WorkbenchContext.Provider>
  );
}

function ArtifactNotice({
  artifact,
  onClose,
}: {
  artifact: CompileArtifact;
  onClose: () => void;
}) {
  return (
    <div className="wb-toasts wb-toasts--bottom">
      <div className="wb-toast">
        <span className="dot" /> {artifact.filename} is ready.
        <button
          type="button"
          className="linkish"
          onClick={() => {
            downloadArtifact(artifact);
            onClose();
          }}
        >
          Save
        </button>
        <button type="button" className="linkish" onClick={onClose}>
          Dismiss
        </button>
      </div>
    </div>
  );
}

function AskDialog({
  options,
  onDone,
}: {
  options: AskOptions;
  onDone: (value: string | null) => void;
}) {
  const [value, setValue] = useState(options.initial ?? "");
  return (
    <div
      className="wb-modal-backdrop"
      onMouseDown={(e) => e.target === e.currentTarget && onDone(null)}
    >
      <form
        className="wb-dialog"
        role="dialog"
        aria-modal="true"
        aria-label={options.title}
        onSubmit={(e) => {
          e.preventDefault();
          onDone(value.trim() || null);
        }}
        onKeyDown={(e) => e.key === "Escape" && onDone(null)}
      >
        <h2>{options.title}</h2>
        <label className="field">
          <span>{options.label}</span>
          {options.multiline ? (
            <textarea
              autoFocus
              rows={4}
              value={value}
              placeholder={options.placeholder}
              onChange={(e) => setValue(e.target.value)}
            />
          ) : (
            <input
              autoFocus
              value={value}
              placeholder={options.placeholder}
              onChange={(e) => setValue(e.target.value)}
            />
          )}
        </label>
        <div className="dialog-actions">
          <button type="button" className="wb-btn" onClick={() => onDone(null)}>
            Cancel
          </button>
          <button
            type="submit"
            className="wb-btn wb-btn--primary"
            disabled={!value.trim()}
          >
            {options.confirm ?? "OK"}
          </button>
        </div>
      </form>
    </div>
  );
}

interface PmNode {
  type?: string;
  text?: string;
  attrs?: { level?: number; id?: string; textAlign?: string };
  content?: PmNode[];
}

const plain = (node: PmNode): string =>
  node.type === "text"
    ? (node.text ?? "")
    : (node.content ?? []).map(plain).join("");

/** Read mode: the page as a reader sees it, nothing editable, nothing in red. */
function ReadView({
  title,
  kind,
  content,
  compact,
}: {
  title: string;
  kind: string;
  content: unknown;
  compact?: boolean;
}) {
  const blocks = ((content as PmNode)?.content ?? []) as PmNode[];
  return (
    <article className={`read-view${compact ? " compact" : ""}`}>
      <p className="eyebrow">{kind}</p>
      <h1>{title}</h1>
      <div className="read-columns">
        {blocks.map((block, i) =>
          block.type === "heading" ? (
            <h2 key={i}>{plain(block)}</h2>
          ) : block.type === "sceneBreak" || block.type === "horizontalRule" ? (
            <p key={i} className="read-break">
              * * *
            </p>
          ) : block.type === "blockquote" ? (
            <blockquote key={i}>{plain(block)}</blockquote>
          ) : (
            <p
              key={i}
              style={
                block.attrs?.textAlign
                  ? { textAlign: block.attrs.textAlign as "center" }
                  : undefined
              }
            >
              {plain(block)}
            </p>
          ),
        )}
      </div>
    </article>
  );
}

/** Word's Navigation pane: headings in the page, else the scenes around it, and the page's marks. */
function NavigationPane({
  documentId,
  editor,
}: {
  documentId?: string;
  editor: Editor | null;
}) {
  const api = useWorkbenchSafe();
  const [, bump] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    if (!editor) return;
    editor.on("update", bump);
    return () => {
      editor.off("update", bump);
    };
  }, [editor]);
  const headings: { text: string; level: number; pos: number }[] = [];
  editor?.state.doc.descendants((node, pos) => {
    if (node.type.name === "heading")
      headings.push({
        text: node.textContent,
        level: node.attrs.level as number,
        pos,
      });
    return node.type.name === "doc";
  });
  const current = api?.documents.find((d) => d.id === documentId);
  const siblings =
    api?.documents
      .filter((d) => d.parent_id === current?.parent_id && d.kind === "scene")
      .sort((a, b) => a.position - b.position) ?? [];
  const pageMarks = (api?.diagnostics?.diagnostics ?? []).filter(
    (m) => m.primary?.document === documentId && !m.suppressed_by,
  );
  return (
    <nav className="nav-pane" aria-label="Navigation">
      <div className="pane-section">Navigation</div>
      {headings.length > 0
        ? headings.map((h, i) => (
            <button
              key={i}
              type="button"
              className="nav-item"
              style={{ paddingLeft: 8 + (h.level - 1) * 12 }}
              onClick={() =>
                editor
                  ?.chain()
                  .focus()
                  .setTextSelection(h.pos + 1)
                  .scrollIntoView()
                  .run()
              }
            >
              {h.text || "Untitled heading"}
            </button>
          ))
        : siblings.map((d) => (
            <button
              key={d.id}
              type="button"
              className={`nav-item${d.id === documentId ? " active" : ""}`}
              onClick={() => api?.openDocument(d.id)}
            >
              {d.title}
            </button>
          ))}
      {pageMarks.length > 0 && (
        <>
          <div className="pane-section">On this page</div>
          {pageMarks.map((m) => (
            <button
              key={m.id}
              type="button"
              className="nav-item"
              onClick={() => {
                api?.selectMark(m.id);
                if (m.primary) api?.jumpTo(m.primary);
              }}
            >
              <span className={`sev-dot sev-${m.severity}`} /> {m.code}
            </button>
          ))}
        </>
      )}
    </nav>
  );
}

function Ruler() {
  return (
    <div className="ruler" aria-hidden="true">
      <div className="ruler-inner">
        {Array.from({ length: 17 }, (_, i) => (
          <span key={i} className={i % 2 === 0 ? "tick major" : "tick"}>
            {i % 2 === 0 ? i / 2 : ""}
          </span>
        ))}
      </div>
    </div>
  );
}

function useWorkbenchSafe() {
  return useContext(WorkbenchContext);
}

function importFormat(name: string): ImportFormat {
  const lower = name.toLowerCase();
  if (lower.endsWith(".docx")) return "docx";
  if (lower.endsWith(".fdx")) return "fdx";
  if (lower.endsWith(".html") || lower.endsWith(".htm")) return "html";
  if (lower.endsWith(".fountain")) return "fountain";
  if (lower.endsWith(".md")) return "markdown";
  return "plain";
}

function toBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let index = 0; index < bytes.length; index += chunk) {
    binary += String.fromCharCode.apply(
      null,
      Array.from(bytes.subarray(index, index + chunk)),
    );
  }
  return btoa(binary);
}
