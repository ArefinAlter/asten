import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  type FormEvent,
  Suspense,
  lazy,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type {
  CompileArtifact,
  CompileFormat,
  CreateProject,
  DocumentKind,
  DocumentSummary,
  ImportFormat,
  Location,
  ProjectId,
  ProjectKind,
  JsonValue,
} from "@asten/contracts";
import { PairScreen } from "./auth/PairScreen";
import { Binder } from "./binder/Binder";
import { Glyph, type GlyphKey } from "./brand/Glyphs";
import { PaletteMark } from "./brand/Palette";
import { ThemeToggle } from "./brand/ThemeToggle";
import { ManuscriptEditor, type Jump } from "./editor/ManuscriptEditor";
import { Inspector } from "./inspector/Inspector";
import { useAnnounceNew, useAnnouncement, useFocusTrap } from "./lib/a11y";
import { ApiError, createAstenClient } from "./lib/asten-client";
import {
  SAVE_DOCUMENT_KEY,
  useMediaQuery,
  useProjectStream,
  useStoredState,
} from "./lib/hooks";
import { emptyDoc } from "./lib/pm";
import { Overview } from "./overview/Overview";
import { ScalesPanel } from "./scales/ScalesPanel";
import type { WorldLens } from "./world/useWorld";

// The overview and the editor are what the desk opens on; every other view
// loads the first time it is visited, so the first paint carries less script.
const Canon = lazy(() =>
  import("./canon/Canon").then((module) => ({ default: module.Canon })),
);
const Cast = lazy(() =>
  import("./cast/Cast").then((module) => ({ default: module.Cast })),
);
const CompileDock = lazy(() =>
  import("./compile/CompileDock").then((module) => ({
    default: module.CompileDock,
  })),
);
const Guests = lazy(() =>
  import("./guests/Guests").then((module) => ({ default: module.Guests })),
);
const History = lazy(() =>
  import("./history/History").then((module) => ({ default: module.History })),
);
const Packs = lazy(() =>
  import("./packs/Packs").then((module) => ({ default: module.Packs })),
);
const Settings = lazy(() =>
  import("./settings/Settings").then((module) => ({
    default: module.Settings,
  })),
);
const Timeline = lazy(() =>
  import("./timeline/Timeline").then((module) => ({
    default: module.Timeline,
  })),
);
const WorldInspector = lazy(() =>
  import("./world/WorldInspector").then((module) => ({
    default: module.WorldInspector,
  })),
);
const WorldWorkspace = lazy(() =>
  import("./world/WorldWorkspace").then((module) => ({
    default: module.WorldWorkspace,
  })),
);

const viewFallback = <p className="status">Opening…</p>;

const client = createAstenClient();

const newMarksPhrase = (count: number) =>
  `${count} new mark${count === 1 ? "" : "s"}`;

/** Siblings that stay live while a drawer or sheet holds focus. */
const KEEP_LIVE = ".scrim, .live-region";

export type View =
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
  | "packs";

const ACTIVITIES: {
  view: View;
  label: string;
  glyph: GlyphKey;
  needsProject: boolean;
}[] = [
  { view: "overview", label: "Overview", glyph: "house", needsProject: false },
  { view: "write", label: "Write", glyph: "scroll", needsProject: true },
  { view: "cast", label: "Cast", glyph: "seated", needsProject: true },
  { view: "canon", label: "Canon", glyph: "feather", needsProject: true },
  { view: "time", label: "Time", glyph: "moon", needsProject: true },
  { view: "world", label: "World", glyph: "hills", needsProject: true },
  { view: "history", label: "History", glyph: "pyramid", needsProject: true },
  { view: "compile", label: "Compile", glyph: "kit", needsProject: true },
  { view: "packs", label: "Packs", glyph: "papyrus", needsProject: false },
  { view: "guests", label: "Guests", glyph: "door", needsProject: false },
  { view: "settings", label: "Settings", glyph: "star", needsProject: true },
];

const VIEWS: ReadonlySet<string> = new Set(
  ACTIVITIES.map((activity) => activity.view),
);

function readHash(): { project?: string; view?: View; document?: string } {
  const match = /^#\/p\/([^/]+)(?:\/([^/]+))?(?:\/([^/]+))?/.exec(
    window.location.hash,
  );
  if (!match) return {};
  const view = match[2] && VIEWS.has(match[2]) ? (match[2] as View) : undefined;
  return {
    project: decodeURIComponent(match[1]),
    view,
    document: match[3] ? decodeURIComponent(match[3]) : undefined,
  };
}

function useAuthGate() {
  const [tick, setTick] = useState(0);
  const status = useQuery({
    queryKey: ["auth-status", tick],
    queryFn: () => client.authStatus(),
    retry: 2,
  });
  const me = useQuery({
    queryKey: ["auth-me", tick],
    queryFn: () => client.me(),
    enabled: Boolean(status.data?.requires_auth),
    retry: false,
  });
  const authenticated = status.data
    ? !status.data.requires_auth || me.isSuccess
    : false;
  const needsPairing =
    Boolean(status.data?.requires_auth) &&
    me.isError &&
    me.error instanceof ApiError &&
    me.error.status === 401;
  // A revoked or expired token surfaces as a 401 on any request; the client
  // forgets it and tells us, and we re-check so the pair screen comes back.
  useEffect(() => {
    const onUnauthorized = () => setTick((value) => value + 1);
    window.addEventListener("asten:unauthorized", onUnauthorized);
    return () =>
      window.removeEventListener("asten:unauthorized", onUnauthorized);
  }, []);
  return {
    status: status.data,
    loading:
      status.isPending || (Boolean(status.data?.requires_auth) && me.isPending),
    authenticated,
    needsPairing,
    unreachable: status.isError,
    refresh: () => setTick((value) => value + 1),
  };
}

function App() {
  const gate = useAuthGate();
  if (gate.unreachable) {
    return (
      <main className="pair-screen">
        <section className="pair-card register">
          <div className="pair-head">
            <PaletteMark />
            <h1>The desk cannot reach its API</h1>
          </div>
          <p className="lede">
            Start the local API (<code>pnpm dev:api</code>) or open the desktop
            app. Nothing of yours lives anywhere but this machine, so there is
            no server to wait for.
          </p>
          <button
            type="button"
            className="btn btn--primary"
            onClick={gate.refresh}
          >
            Try again
          </button>
        </section>
      </main>
    );
  }
  if (gate.loading || !gate.status) {
    return (
      <main className="pair-screen">
        <p className="status">Opening the desk…</p>
      </main>
    );
  }
  if (!gate.authenticated || gate.needsPairing) {
    return <PairScreen status={gate.status} onPaired={gate.refresh} />;
  }
  return <Desk />;
}

function Desk() {
  const initial = useMemo(() => readHash(), []);
  const [selectedProjectId, setSelectedProjectId] = useState<
    ProjectId | undefined
  >(initial.project);
  const [selectedDocumentId, setSelectedDocumentId] = useState<
    string | undefined
  >(initial.document);
  const [view, setView] = useState<View>(initial.view ?? "overview");
  const [worldLens, setWorldLens] = useState<WorldLens>("wall");
  const [selectedWorldId, setSelectedWorldId] = useState<string>();
  const [selectedEntityKey, setSelectedEntityKey] = useState<string>();
  const [selectedMarkId, setSelectedMarkId] = useState<string>();
  const [jump, setJump] = useState<Jump>();
  const [isCreating, setIsCreating] = useState(false);
  const [pageView, setPageView] = useStoredState<"on" | "off">(
    "asten-page-view",
    "off",
  );
  const [trackChanges, setTrackChanges] = useState(false);
  const [showBlame, setShowBlame] = useState(false);
  const [findQuery, setFindQuery] = useState("");
  const [artifact, setArtifact] = useState<CompileArtifact>();
  const [scalesCollapsed, setScalesCollapsed] = useStoredState<
    "open" | "closed"
  >("asten-scales", "open");
  const [saveError, setSaveError] = useState<string>();
  const queryClient = useQueryClient();
  const connected = useProjectStream(selectedProjectId);

  // Tablets fold the inspector into a sheet; phones stack the desk in one
  // column with the binder as a drawer. The classes drive the stylesheet.
  const compact = useMediaQuery("(max-width: 1279px)");
  const narrow = useMediaQuery("(max-width: 959px)");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [inspectorOpen, setInspectorOpen] = useState(false);
  useEffect(() => {
    if (!narrow) {
      setSidebarOpen(false);
      return;
    }
    // Choosing anything closes the drawer. With no project open it stays
    // open, so the project list is reachable on a phone.
    setSidebarOpen(!selectedProjectId);
  }, [narrow, view, selectedDocumentId, selectedProjectId]);
  useEffect(() => {
    if (!compact) setInspectorOpen(false);
  }, [compact]);

  // Keep the address in step with the desk, and the desk with the address
  // (back, forward, a hand-edited hash).
  useEffect(() => {
    const parts = selectedProjectId
      ? [`#/p/${selectedProjectId}`, view, selectedDocumentId ?? ""]
      : ["#/"];
    const next = parts.filter(Boolean).join("/");
    if (window.location.hash !== next)
      window.history.replaceState(null, "", next);
  }, [selectedProjectId, view, selectedDocumentId]);
  useEffect(() => {
    const onHashChange = () => {
      const target = readHash();
      if (!target.project) return;
      setSelectedProjectId(target.project as ProjectId);
      if (target.view) setView(target.view);
      setSelectedDocumentId(target.document);
    };
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => client.listProjects(),
  });
  // The binder, overview, timeline and history read summaries (no bodies);
  // only the open page fetches its body, and the change stream refreshes
  // that one query when the document moves.
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

  // On small desks the binder drawer and the inspector sheet are dialogs:
  // focus stays inside, the rest of the shell is inert, Escape closes, and
  // focus returns to the toggle. One polite live region serves the desk.
  const sidebarRef = useRef<HTMLElement>(null);
  const inspectorRef = useRef<HTMLElement>(null);
  const binderToggleRef = useRef<HTMLButtonElement>(null);
  const inspectorToggleRef = useRef<HTMLButtonElement>(null);
  const drawerOpen = narrow && sidebarOpen;
  const sheetOpen = compact && inspectorOpen;
  useFocusTrap(sidebarRef, drawerOpen, {
    onClose: () => setSidebarOpen(false),
    returnTo: binderToggleRef,
    keep: KEEP_LIVE,
  });
  useFocusTrap(inspectorRef, sheetOpen, {
    onClose: () => setInspectorOpen(false),
    returnTo: inspectorToggleRef,
    keep: KEEP_LIVE,
  });
  const announcement = useAnnouncement();
  useAnnounceNew(
    diagnostics.data?.diagnostics,
    selectedProjectId,
    newMarksPhrase,
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
      // The response is the new body; no refetch of it is needed. The
      // summaries are small, so they refresh in full.
      queryClient.setQueryData(["document", saved.project_id, saved.id], saved);
      await queryClient.invalidateQueries({
        queryKey: ["documents", selectedProjectId],
      });
    },
    onError: (error, input) => {
      setSaveError(error.message);
      // A conflict means someone else moved the page: fetch their head so
      // "reload theirs" has something to reload.
      if (error instanceof ApiError && error.status === 409) {
        void queryClient.invalidateQueries({
          queryKey: ["document", selectedProjectId, input.documentId],
        });
      }
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
    onSuccess: setArtifact,
  });
  const importDocument = useMutation({
    mutationFn: async (file: File) => {
      const format = importFormat(file.name);
      const buffer = await file.arrayBuffer();
      const body =
        format === "docx"
          ? toBase64(new Uint8Array(buffer))
          : new TextDecoder().decode(buffer);
      return client.importDocument(selectedProjectId!, {
        filename: file.name,
        format,
        body,
        encoding: format === "docx" ? "base64" : "utf-8",
      });
    },
    onSuccess: async (document) => {
      setSelectedDocumentId(document.id);
      setView("write");
      await queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });

  const selectedProject = projects.data?.find(
    (project) => project.id === selectedProjectId,
  );
  const selectedSummary = useMemo(
    () =>
      documents.data?.find((document) => document.id === selectedDocumentId),
    [documents.data, selectedDocumentId],
  );
  const selectedDocument =
    openDocument.data?.id === selectedDocumentId
      ? openDocument.data
      : undefined;
  const marks = diagnostics.data?.diagnostics ?? [];
  const openMarks = marks.filter((mark) => !mark.suppressed_by);

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
    [setSelectedDocumentId, setView],
  );

  function selectProject(id: ProjectId) {
    setSelectedProjectId(id);
    setSelectedDocumentId(undefined);
    setSelectedWorldId(undefined);
    setSelectedEntityKey(undefined);
    setSelectedMarkId(undefined);
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

  function selectDocument(document: DocumentSummary) {
    setSelectedDocumentId(document.id);
    setView("write");
  }

  function openEntity(key: string) {
    setSelectedEntityKey(key);
    setView("cast");
  }

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "f" &&
        view === "write"
      ) {
        event.preventDefault();
        document.querySelector<HTMLInputElement>(".find-input")?.focus();
      }
      if (event.key === "Escape") {
        setSidebarOpen(false);
        setInspectorOpen(false);
      }
      if ((event.ctrlKey || event.metaKey) && event.key === "j") {
        event.preventDefault();
        setScalesCollapsed(scalesCollapsed === "open" ? "closed" : "open");
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [view, scalesCollapsed, setScalesCollapsed]);

  const errorCount = openMarks.filter(
    (mark) => mark.severity === "error",
  ).length;
  const warningCount = openMarks.filter(
    (mark) => mark.severity === "warning",
  ).length;

  return (
    <div
      className={`app-shell view-${view}${scalesCollapsed === "closed" ? " scales-closed" : ""}${compact ? " compact" : ""}${narrow ? " narrow" : ""}${sidebarOpen ? " sidebar-open" : ""}${inspectorOpen ? " inspector-open" : ""}`}
    >
      <a
        className="skip-link"
        href="#workspace"
        onClick={(event) => {
          event.preventDefault();
          window.document.getElementById("workspace")?.focus();
        }}
      >
        Skip to the page
      </a>
      <div className="sr-only live-region" role="status" aria-live="polite">
        {announcement}
      </div>
      <header className="titlebar">
        <span className="brand">
          <button
            type="button"
            ref={binderToggleRef}
            className={`shell-toggle binder-toggle${sidebarOpen ? " active" : ""}`}
            aria-label={sidebarOpen ? "Close the binder" : "Open the binder"}
            aria-expanded={sidebarOpen}
            onClick={() => {
              setInspectorOpen(false);
              setSidebarOpen((value) => !value);
            }}
          >
            ≡
          </button>
          <PaletteMark />
          <span className="wordmark">Asten</span>
        </span>
        <span className="title-crumb">
          {selectedProject?.name ?? "Project library"}
          {selectedSummary && view === "write" ? (
            <span className="crumb-doc"> · {selectedSummary.title}</span>
          ) : null}
        </span>
        <div className="titlebar-actions">
          <input
            className="find-input"
            placeholder="Find in the page"
            value={findQuery}
            onChange={(event) => setFindQuery(event.target.value)}
            disabled={view !== "write"}
            aria-label="Find in the page"
          />
          <button
            type="button"
            className={pageView === "on" ? "active" : ""}
            onClick={() => setPageView(pageView === "on" ? "off" : "on")}
            title="Letter page view"
            disabled={view !== "write"}
          >
            Page
          </button>
          <button
            type="button"
            className={trackChanges ? "active" : ""}
            onClick={() => setTrackChanges((value) => !value)}
            title="Track changes"
            disabled={view !== "write"}
          >
            Track
          </button>
          <button
            type="button"
            className={showBlame ? "active" : ""}
            onClick={() => setShowBlame((value) => !value)}
            title="Who wrote each line"
            disabled={view !== "write"}
          >
            Blame
          </button>
          <ThemeToggle />
          <button
            type="button"
            ref={inspectorToggleRef}
            className={`shell-toggle inspector-toggle${inspectorOpen ? " active" : ""}`}
            aria-label={
              inspectorOpen ? "Close the inspector" : "Open the inspector"
            }
            aria-expanded={inspectorOpen}
            onClick={() => {
              setSidebarOpen(false);
              setInspectorOpen((value) => !value);
            }}
          >
            <Glyph name="eye" />
          </button>
        </div>
      </header>
      {((narrow && sidebarOpen) || (compact && inspectorOpen)) && (
        <button
          type="button"
          className="scrim"
          aria-label="Close the panel"
          onClick={() => {
            setSidebarOpen(false);
            setInspectorOpen(false);
          }}
        />
      )}

      <nav className="activitybar" aria-label="Workspace views">
        {ACTIVITIES.map((activity) => (
          <button
            key={activity.view}
            type="button"
            title={activity.label}
            aria-label={activity.label}
            className={view === activity.view ? "selected" : ""}
            disabled={activity.needsProject && !selectedProject}
            onClick={() => setView(activity.view)}
          >
            <Glyph name={activity.glyph} />
            <span className="activity-label">{activity.label}</span>
          </button>
        ))}
      </nav>

      <aside
        className="sidebar"
        ref={sidebarRef}
        inert={narrow && !sidebarOpen}
        role={drawerOpen ? "dialog" : undefined}
        aria-modal={drawerOpen || undefined}
        aria-label="Projects and binder"
        tabIndex={-1}
      >
        <div className="pane-heading">
          <span className="mono">Projects</span>
          <button
            type="button"
            className="linkish"
            onClick={() => setIsCreating(true)}
          >
            New
          </button>
        </div>
        {projects.isPending && <p className="status">Loading…</p>}
        {projects.error && <p className="error">{projects.error.message}</p>}
        <nav className="tree" aria-label="Projects">
          {projects.data?.map((project) => (
            <button
              className={project.id === selectedProjectId ? "selected" : ""}
              key={project.id}
              onClick={() => selectProject(project.id)}
              type="button"
            >
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
                className="btn btn--small btn--primary"
                disabled={createProject.isPending}
              >
                Create
              </button>
              <button
                type="button"
                className="btn btn--small btn--ghost"
                onClick={() => setIsCreating(false)}
              >
                Cancel
              </button>
            </div>
          </form>
        )}
        {selectedProject && (
          <>
            <div className="pane-heading">
              <span className="mono">Binder</span>
            </div>
            <Binder
              projectId={selectedProject.id}
              documents={documents.data ?? []}
              selectedId={selectedDocumentId}
              marks={openMarks}
              onSelect={selectDocument}
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
                  window.confirm(
                    `Delete “${document.title}”? Its history goes with it.`,
                  )
                )
                  deleteDocument.mutate(document.id);
              }}
            />
          </>
        )}
      </aside>

      <main
        id="workspace"
        tabIndex={-1}
        className={`workspace${view === "world" ? " world-open" : ""}`}
      >
        <Suspense fallback={viewFallback}>
          {!selectedProject ? (
            view === "guests" ? (
              <Guests />
            ) : view === "packs" ? (
              <Packs />
            ) : projects.data?.length === 0 ? (
              <section className="empty-state">
                <PaletteMark className="hero-mark" />
                <h1>A writing desk with a memory.</h1>
                <p>Your words stay black. Asten only ever writes in red.</p>
                <ol className="first-steps">
                  <li>Create a project.</li>
                  <li>Write a scene.</li>
                  <li>Settle a fact when it matters.</li>
                </ol>
                <button
                  type="button"
                  className="btn btn--primary"
                  onClick={() => {
                    setIsCreating(true);
                    if (narrow) setSidebarOpen(true);
                  }}
                >
                  Create a project
                </button>
              </section>
            ) : (
              <section className="empty-state">
                <PaletteMark className="hero-mark" />
                <h1>A writing desk with a memory.</h1>
                <p>
                  Select a project or create one. Your words stay black. Asten
                  only ever writes in red.
                </p>
                {narrow && (
                  <button
                    type="button"
                    className="btn btn--primary"
                    onClick={() => setSidebarOpen(true)}
                  >
                    Open the project list
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
              documents={documents.data ?? []}
              onJump={jumpTo}
            />
          ) : view === "canon" ? (
            <Canon
              projectId={selectedProject.id}
              documents={documents.data ?? []}
              entities={entities.data ?? []}
              snapshot={diagnostics.data}
            />
          ) : view === "time" ? (
            <Timeline
              projectId={selectedProject.id}
              documents={documents.data ?? []}
              entities={entities.data ?? []}
              onOpenDocument={(id) =>
                jumpTo({ document: id, start: 0, end: 0, quote: "", label: "" })
              }
            />
          ) : view === "history" ? (
            <History
              projectId={selectedProject.id}
              documents={documents.data ?? []}
              documentId={selectedDocumentId}
              onSelectDocument={setSelectedDocumentId}
            />
          ) : view === "guests" ? (
            <Guests />
          ) : view === "settings" ? (
            <Settings
              projectId={selectedProject.id}
              snapshot={diagnostics.data}
            />
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
          ) : view === "write" ? (
            <section className="write-pane">
              {saveError && (
                <p className="error banner" role="alert">
                  {saveError}
                </p>
              )}
              {selectedDocument ? (
                <ManuscriptEditor
                  key={selectedDocument.id}
                  projectId={selectedProject.id}
                  document={selectedDocument}
                  diagnostics={marks}
                  entities={entities.data ?? []}
                  pageView={pageView === "on"}
                  trackChanges={trackChanges}
                  findQuery={findQuery}
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
                  onSelectMark={setSelectedMarkId}
                  onSelectEntity={openEntity}
                />
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
                    Add a scene in the binder, or import a manuscript under
                    Compile. The world tools are not the gate.
                  </p>
                </section>
              )}
            </section>
          ) : (
            <Overview
              project={selectedProject}
              documents={documents.data ?? []}
              entities={entities.data ?? []}
              snapshot={diagnostics.data}
              onGo={setView}
              onOpenDocument={(document) => selectDocument(document)}
            />
          )}
        </Suspense>
      </main>

      <aside
        className="inspector"
        ref={inspectorRef}
        inert={compact && !inspectorOpen}
        role={sheetOpen ? "dialog" : undefined}
        aria-modal={sheetOpen || undefined}
        aria-label="Inspector"
        tabIndex={-1}
      >
        {selectedProject && view === "world" ? (
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
            documents={documents.data ?? []}
            entityKey={selectedEntityKey}
            marks={marks}
            selectedMarkId={selectedMarkId}
            onJump={jumpTo}
            onOpenEntity={openEntity}
            onSelectMark={setSelectedMarkId}
          />
        ) : (
          <div className="inspector-empty">
            <p className="hint">
              The inspector shows what is settled about the name under your
              cursor, the page's notes, and the mark you selected.
            </p>
          </div>
        )}
      </aside>

      {selectedProject && (
        <ScalesPanel
          projectId={selectedProject.id}
          snapshot={diagnostics.data}
          loading={diagnostics.isPending}
          documents={documents.data ?? []}
          selectedMarkId={selectedMarkId}
          onSelectMark={setSelectedMarkId}
          onJump={jumpTo}
          collapsed={scalesCollapsed === "closed"}
          onToggleCollapsed={() =>
            setScalesCollapsed(scalesCollapsed === "open" ? "closed" : "open")
          }
        />
      )}

      <footer className="statusbar">
        <span
          className={`live ${connected ? "on" : ""}`}
          title={
            connected
              ? "Live: changes stream from the API"
              : "Not connected to the change stream"
          }
        >
          {connected ? "live" : "offline"}
        </span>
        <span className="trivia">Local first · no account · no sync</span>
        {selectedProject && diagnostics.data ? (
          <span className="mono">
            {errorCount} error{errorCount === 1 ? "" : "s"} · {warningCount}{" "}
            warning{warningCount === 1 ? "" : "s"} · rev{" "}
            {diagnostics.data.revision.slice(0, 8)}
          </span>
        ) : null}
        <span className="trivia">Designed and built by Verne Ltd.</span>
      </footer>
    </div>
  );
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

export default App;

/** Base64 for a file of any size; spreading the bytes into one call overflows. */
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
