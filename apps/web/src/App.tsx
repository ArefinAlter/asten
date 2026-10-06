import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useMemo, useState } from "react";
import type {
  CompileArtifact,
  CompileFormat,
  CreateProject,
  DocumentKind,
  ImportFormat,
  MarginDocument,
  ProjectId,
  ProjectKind,
} from "@asten/contracts";
import { Binder } from "./binder/Binder";
import { Cast } from "./cast/Cast";
import { CompileDock } from "./compile/CompileDock";
import { ManuscriptEditor } from "./editor/ManuscriptEditor";
import { createMarginClient } from "./lib/margin-client";
import { emptyDoc } from "./lib/pm";
import { Packs } from "./packs/Packs";
import { Timeline } from "./timeline/Timeline";
import { WorldInspector } from "./world/WorldInspector";
import { WorldWorkspace } from "./world/WorldWorkspace";
import type { WorldLens } from "./world/useWorld";
import "./margin.css";

const client = createMarginClient();
type View =
  | "overview"
  | "write"
  | "world"
  | "cast"
  | "time"
  | "compile"
  | "packs";

function App() {
  const [selectedProjectId, setSelectedProjectId] = useState<ProjectId>();
  const [selectedDocumentId, setSelectedDocumentId] = useState<string>();
  const [view, setView] = useState<View>("overview");
  const [worldLens, setWorldLens] = useState<WorldLens>("wall");
  const [selectedWorldId, setSelectedWorldId] = useState<string>();
  const [isCreating, setIsCreating] = useState(false);
  const [pageView, setPageView] = useState(false);
  const [trackChanges, setTrackChanges] = useState(false);
  const [findQuery, setFindQuery] = useState("");
  const [artifact, setArtifact] = useState<CompileArtifact>();
  const queryClient = useQueryClient();

  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => client.listProjects(),
  });
  const documents = useQuery({
    queryKey: ["documents", selectedProjectId],
    queryFn: () => client.listDocuments(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const facts = useQuery({
    queryKey: ["facts", selectedProjectId],
    queryFn: () => client.listFacts(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const diagnostics = useQuery({
    queryKey: ["diagnostics", selectedProjectId],
    queryFn: () => client.listDiagnostics(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const entities = useQuery({
    queryKey: ["entities", selectedProjectId],
    queryFn: () => client.listEntities(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const events = useQuery({
    queryKey: ["events", selectedProjectId],
    queryFn: () => client.listEvents(selectedProjectId!),
    enabled: Boolean(selectedProjectId),
  });
  const packs = useQuery({
    queryKey: ["packs"],
    queryFn: () => client.listPacks(),
  });
  const comments = useQuery({
    queryKey: ["comments", selectedProjectId, selectedDocumentId],
    queryFn: () => client.listComments(selectedProjectId!, selectedDocumentId!),
    enabled: Boolean(selectedProjectId && selectedDocumentId),
  });
  const snapshots = useQuery({
    queryKey: ["snapshots", selectedProjectId, selectedDocumentId],
    queryFn: () => client.listSnapshots(selectedProjectId!, selectedDocumentId!),
    enabled: Boolean(selectedProjectId && selectedDocumentId),
  });

  const createProject = useMutation({
    mutationFn: (input: CreateProject) => client.createProject(input),
    onSuccess: async (project) => {
      setSelectedProjectId(project.id);
      setIsCreating(false);
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
      }),
    onSuccess: async (document) => {
      setSelectedDocumentId(document.id);
      setView("write");
      await queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
  const updateDocument = useMutation({
    mutationFn: (input: {
      documentId: string;
      content_json?: unknown;
      exclude_from_compile?: boolean;
    }) =>
      client.updateDocument(selectedProjectId!, input.documentId, {
        content_json: input.content_json,
        exclude_from_compile: input.exclude_from_compile,
        actor: "author",
      }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["documents"] }),
        queryClient.invalidateQueries({ queryKey: ["diagnostics"] }),
      ]);
    },
  });
  const createFact = useMutation({
    mutationFn: (input: {
      entity_key: string;
      predicate: string;
      value: string;
    }) =>
      client.createFact({
        project_id: selectedProjectId!,
        ...input,
        status: "confirmed",
        modality: "established",
      }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["facts"] }),
        queryClient.invalidateQueries({ queryKey: ["diagnostics"] }),
      ]);
    },
  });
  const createEntity = useMutation({
    mutationFn: (input: {
      name: string;
      kind: "character" | "place" | "event" | "item" | "rule" | "source" | "other";
      notes: string;
    }) =>
      client.createEntity({
        project_id: selectedProjectId!,
        ...input,
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["entities"] });
      await queryClient.invalidateQueries({ queryKey: ["facts"] });
    },
  });
  const createEvent = useMutation({
    mutationFn: (input: {
      title: string;
      story_time?: string;
      reading_order?: number;
      notes?: string;
      entity_id?: string | null;
      document_id?: string | null;
    }) =>
      client.createEvent({
        project_id: selectedProjectId!,
        ...input,
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["events"] });
      await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
    },
  });
  const createComment = useMutation({
    mutationFn: (body: string) =>
      client.createComment(selectedProjectId!, selectedDocumentId!, { body }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["comments"] });
    },
  });
  const createSnapshot = useMutation({
    mutationFn: () =>
      client.createSnapshot(selectedProjectId!, selectedDocumentId!, "Named"),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["snapshots"] });
    },
  });
  const restoreSnapshot = useMutation({
    mutationFn: (id: string) =>
      client.restoreSnapshot(selectedProjectId!, id),
    onSuccess: async (document) => {
      setSelectedDocumentId(document.id);
      await queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
  const suppress = useMutation({
    mutationFn: (input: { rule_code: string; document_id: string }) =>
      client.createSuppression(selectedProjectId!, {
        ...input,
        reason: "Author suppressed",
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
    },
  });
  const togglePack = useMutation({
    mutationFn: (input: { id: string; enabled: boolean }) =>
      client.updatePack(input.id, input.enabled),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["packs"] });
      await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
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
          ? btoa(String.fromCharCode(...new Uint8Array(buffer)))
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
  const selectedDocument = useMemo(
    () => documents.data?.find((document) => document.id === selectedDocumentId),
    [documents.data, selectedDocumentId],
  );

  function submitProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createProject.mutate({
      name: String(form.get("name") ?? ""),
      kind: String(form.get("kind") ?? "novel") as ProjectKind,
    });
  }

  function submitFact(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createFact.mutate({
      entity_key: String(form.get("entity_key") ?? ""),
      predicate: String(form.get("predicate") ?? ""),
      value: String(form.get("value") ?? ""),
    });
    event.currentTarget.reset();
  }

  function submitComment(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createComment.mutate(String(form.get("body") ?? ""));
    event.currentTarget.reset();
  }

  function selectDocument(document: MarginDocument) {
    setSelectedDocumentId(document.id);
    setView("write");
  }

  return (
    <div className={`app-shell${view === "world" ? " world-mode" : ""}`}>
      <header className="titlebar">
        <strong>Asten</strong>
        <span>{selectedProject?.name ?? "Project library"}</span>
        <div className="titlebar-actions">
          <input
            className="find-input"
            placeholder="Find in the page"
            value={findQuery}
            onChange={(event) => setFindQuery(event.target.value)}
            disabled={view !== "write"}
          />
          <button
            type="button"
            className={pageView ? "active" : ""}
            onClick={() => setPageView((value) => !value)}
          >
            Page
          </button>
          <button
            type="button"
            className={trackChanges ? "active" : ""}
            onClick={() => setTrackChanges((value) => !value)}
          >
            Track
          </button>
          <button type="button" onClick={() => setView("compile")}>
            Export
          </button>
        </div>
      </header>

      <aside className="activitybar" aria-label="Workspace views">
        <button
          type="button"
          title="Write"
          className={view === "write" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => setView("write")}
        >
          W
        </button>
        <button
          type="button"
          title="World wall"
          className={view === "world" && worldLens === "wall" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => {
            setWorldLens("wall");
            setView("world");
          }}
        >
          ✦
        </button>
        <button
          type="button"
          title="Cast"
          className={view === "cast" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => setView("cast")}
        >
          C
        </button>
        <button
          type="button"
          title="Time"
          className={view === "time" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => setView("time")}
        >
          T
        </button>
        <button
          type="button"
          title="Map"
          className={view === "world" && worldLens === "atlas" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => {
            setWorldLens("atlas");
            setView("world");
          }}
        >
          M
        </button>
        <button
          type="button"
          title="Dossier"
          className={
            view === "world" && worldLens === "dossier" ? "selected" : ""
          }
          disabled={!selectedProject}
          onClick={() => {
            setWorldLens("dossier");
            setView("world");
          }}
        >
          D
        </button>
        <button
          type="button"
          title="Compile"
          className={view === "compile" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => setView("compile")}
        >
          ⬇
        </button>
        <button
          type="button"
          title="Packs"
          className={view === "packs" ? "selected" : ""}
          disabled={!selectedProject}
          onClick={() => setView("packs")}
        >
          ◇
        </button>
      </aside>

      <aside className="sidebar">
        <div className="pane-heading">
          <span>Projects</span>
          <button type="button" onClick={() => setIsCreating(true)}>
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
              onClick={() => {
                setSelectedProjectId(project.id);
                setSelectedDocumentId(undefined);
                setSelectedWorldId(undefined);
                setView("overview");
              }}
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
            <div>
              <button type="submit" disabled={createProject.isPending}>
                Create
              </button>
              <button type="button" onClick={() => setIsCreating(false)}>
                Cancel
              </button>
            </div>
          </form>
        )}
        {selectedProject && (
          <>
            <div className="pane-heading">
              <span>Binder</span>
            </div>
            <Binder
              documents={documents.data ?? []}
              selectedId={selectedDocumentId}
              onSelect={selectDocument}
              onCreate={(input) => createDocument.mutate(input)}
              creating={createDocument.isPending}
              onToggleCompile={(document) =>
                updateDocument.mutate({
                  documentId: document.id,
                  exclude_from_compile: !document.exclude_from_compile,
                })
              }
            />
          </>
        )}
      </aside>

      <main className={`workspace${view === "world" ? " world-open" : ""}`}>
        {!selectedProject ? (
          <section className="empty-state">
            <h1>Asten</h1>
            <p>The environment that notices. Select a project or create one.</p>
          </section>
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
            entities={entities.data ?? []}
            facts={facts.data ?? []}
            creating={createEntity.isPending}
            onCreate={(input) => createEntity.mutate(input)}
          />
        ) : view === "time" ? (
          <Timeline
            events={events.data ?? []}
            entities={entities.data ?? []}
            documents={documents.data ?? []}
            creating={createEvent.isPending}
            onCreate={(input) => createEvent.mutate(input)}
          />
        ) : view === "compile" ? (
          <CompileDock
            compiling={compile.isPending}
            artifact={artifact}
            onCompile={(format) => compile.mutate(format)}
            onImportFile={(file) => importDocument.mutate(file)}
          />
        ) : view === "packs" ? (
          <Packs
            packs={packs.data ?? []}
            onToggle={(id, enabled) => togglePack.mutate({ id, enabled })}
          />
        ) : view === "write" ? (
          <section className="write-pane">
            {selectedDocument ? (
              <ManuscriptEditor
                key={selectedDocument.id}
                document={selectedDocument}
                pageView={pageView}
                trackChanges={trackChanges}
                findQuery={findQuery}
                saving={updateDocument.isPending}
                onSave={(content_json) =>
                  updateDocument.mutate({
                    documentId: selectedDocument.id,
                    content_json,
                  })
                }
              />
            ) : (
              <section className="editor-placeholder">
                <p className="eyebrow">Binder</p>
                <h1>Start a scene</h1>
                <p>Add a node in the binder. World tools are not the gate.</p>
              </section>
            )}
          </section>
        ) : (
          <section className="editor-placeholder">
            <p className="eyebrow">{selectedProject.kind}</p>
            <h1>{selectedProject.name}</h1>
            <p>
              Write, pin a scrap, or name a character. None of those is required
              first. Asten notices contradictions after you settle them.
            </p>
          </section>
        )}
      </main>

      <aside className="inspector">
        {selectedProject && view === "world" ? (
          <WorldInspector
            projectId={selectedProject.id}
            selectedId={selectedWorldId}
          />
        ) : (
          <>
            <div className="pane-heading">Canon facts</div>
            {!selectedProject && (
              <p className="status">Select a project to add facts.</p>
            )}
            {selectedProject && (
              <>
                <ul className="fact-list">
                  {facts.data?.map((fact) => (
                    <li key={fact.id}>
                      <strong>
                        {fact.entity_key}.{fact.predicate}
                      </strong>
                      <span>{fact.value}</span>
                    </li>
                  ))}
                </ul>
                <form className="stacked-form" onSubmit={submitFact}>
                  <input name="entity_key" placeholder="mira" required />
                  <input name="predicate" placeholder="hair_color" required />
                  <input name="value" placeholder="auburn" required />
                  <button type="submit" disabled={createFact.isPending}>
                    Confirm fact
                  </button>
                </form>
              </>
            )}
            {selectedDocument && (
              <>
                <div className="pane-heading">Comments</div>
                <ul className="fact-list">
                  {comments.data?.map((comment) => (
                    <li key={comment.id}>{comment.body}</li>
                  ))}
                </ul>
                <form className="stacked-form" onSubmit={submitComment}>
                  <input name="body" placeholder="A note for later" required />
                  <button type="submit" disabled={createComment.isPending}>
                    Leave a comment
                  </button>
                </form>
                <div className="pane-heading">Snapshots</div>
                <button
                  type="button"
                  onClick={() => createSnapshot.mutate()}
                  disabled={createSnapshot.isPending}
                >
                  Snapshot this revision
                </button>
                <ul className="fact-list">
                  {snapshots.data?.map((snapshot) => (
                    <li key={snapshot.id}>
                      <span>
                        r{snapshot.revision} {snapshot.label ?? ""}
                      </span>
                      <button
                        type="button"
                        className="linkish"
                        onClick={() => restoreSnapshot.mutate(snapshot.id)}
                      >
                        Restore
                      </button>
                    </li>
                  ))}
                </ul>
              </>
            )}
          </>
        )}
      </aside>

      <section className="problems">
        <div className="pane-heading">
          <span>Problems</span>
          <span>{diagnostics.data?.length ?? 0}</span>
        </div>
        {!diagnostics.data?.length && <p className="status">No diagnostics.</p>}
        <ul className="problem-list">
          {diagnostics.data?.map((item) => (
            <li key={item.id} className={item.severity}>
              <code>{item.code}</code>
              <span>{item.message}</span>
              <button
                type="button"
                className="linkish"
                onClick={() =>
                  suppress.mutate({
                    rule_code: item.code,
                    document_id: item.source.document_id,
                  })
                }
              >
                Suppress
              </button>
            </li>
          ))}
        </ul>
      </section>

      <footer className="statusbar">
        <span>Local first</span>
        <span>{diagnostics.data?.length ?? 0} diagnostics</span>
        <span>Asten 0.1.0</span>
      </footer>
    </div>
  );
}

function importFormat(name: string): ImportFormat {
  const lower = name.toLowerCase();
  if (lower.endsWith(".docx")) return "docx";
  if (lower.endsWith(".html") || lower.endsWith(".htm")) return "html";
  if (lower.endsWith(".fountain")) return "fountain";
  if (lower.endsWith(".md")) return "markdown";
  return "plain";
}

export default App;
