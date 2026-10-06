import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useMemo, useState } from "react";
import type {
  CreateProject,
  DocumentKind,
  MarginDocument,
  ProjectId,
  ProjectKind,
} from "@margin/contracts";
import { createMarginClient } from "./lib/margin-client";
import { WorldInspector } from "./world/WorldInspector";
import { WorldWorkspace } from "./world/WorldWorkspace";
import type { WorldLens } from "./world/useWorld";
import "./margin.css";

const client = createMarginClient();

function documentText(content: unknown): string {
  if (
    content &&
    typeof content === "object" &&
    "text" in content &&
    typeof content.text === "string"
  ) {
    return content.text;
  }
  return "";
}

function App() {
  const [selectedProjectId, setSelectedProjectId] = useState<ProjectId>();
  const [selectedDocumentId, setSelectedDocumentId] = useState<string>();
  const [view, setView] = useState<"overview" | "write" | "world">("overview");
  const [worldLens, setWorldLens] = useState<WorldLens>("wall");
  const [selectedWorldId, setSelectedWorldId] = useState<string>();
  const [isCreating, setIsCreating] = useState(false);
  const [draftText, setDraftText] = useState("");
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
      text: string;
    }) =>
      client.createDocument({
        project_id: selectedProjectId!,
        kind: input.kind,
        title: input.title,
        content_json: { text: input.text },
      }),
    onSuccess: async (document) => {
      setSelectedDocumentId(document.id);
      setDraftText(documentText(document.content_json));
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["documents"] }),
        queryClient.invalidateQueries({ queryKey: ["diagnostics"] }),
      ]);
    },
  });
  const updateDocument = useMutation({
    mutationFn: (input: { documentId: string; text: string }) =>
      client.updateDocument(selectedProjectId!, input.documentId, {
        content_json: { text: input.text },
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

  function submitDocument(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createDocument.mutate({
      title: String(form.get("title") ?? ""),
      kind: String(form.get("kind") ?? "scene") as DocumentKind,
      text: String(form.get("text") ?? ""),
    });
    event.currentTarget.reset();
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

  function selectDocument(document: MarginDocument) {
    setSelectedDocumentId(document.id);
    setDraftText(documentText(document.content_json));
    setView("write");
  }

  return (
    <div className={`app-shell${view === "world" ? " world-mode" : ""}`}>
      <header className="titlebar">
        <strong>Margin</strong>
        <span>{selectedProject?.name ?? "Project library"}</span>
        <div className="titlebar-actions">
          <button type="button" disabled>
            Sync
          </button>
          <button type="button" disabled>
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
          className={
            view === "world" && worldLens === "wall" ? "selected" : ""
          }
          disabled={!selectedProject}
          onClick={() => {
            setWorldLens("wall");
            setView("world");
          }}
        >
          ✦
        </button>
        <button type="button" title="Graph" disabled>
          G
        </button>
        <button type="button" title="Time" disabled>
          T
        </button>
        <button
          type="button"
          title="Map"
          className={
            view === "world" && worldLens === "atlas" ? "selected" : ""
          }
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
                setDraftText("");
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
            {createProject.error && (
              <p className="error">{createProject.error.message}</p>
            )}
          </form>
        )}
        {selectedProject && (
          <>
            <div className="pane-heading">
              <span>Documents</span>
            </div>
            <nav className="tree" aria-label="Documents">
              {documents.data?.map((document) => (
                <button
                  className={
                    document.id === selectedDocumentId ? "selected" : ""
                  }
                  key={document.id}
                  onClick={() => selectDocument(document)}
                  type="button"
                >
                  <span>{document.title}</span>
                  <small>{document.kind}</small>
                </button>
              ))}
            </nav>
          </>
        )}
      </aside>

      <main className={`workspace${view === "world" ? " world-open" : ""}`}>
        {!selectedProject ? (
          <section className="empty-state">
            <h1>Margin</h1>
            <p>Select a project or create one to begin.</p>
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
        ) : view === "write" ? (
          <section className="write-pane">
            <div className="tabs">
              <button type="button" onClick={() => setView("overview")}>
                Overview
              </button>
              <button type="button" className="active">
                Write
              </button>
              <button
                type="button"
                onClick={() => {
                  setWorldLens("wall");
                  setView("world");
                }}
              >
                World
              </button>
            </div>
            {selectedDocument ? (
              <div className="editor-placeholder write-editor">
                <p className="eyebrow">{selectedDocument.kind}</p>
                <h1>{selectedDocument.title}</h1>
                <textarea
                  aria-label="Manuscript text"
                  value={draftText}
                  onChange={(event) => setDraftText(event.target.value)}
                />
                <button
                  type="button"
                  disabled={updateDocument.isPending}
                  onClick={() =>
                    updateDocument.mutate({
                      documentId: selectedDocument.id,
                      text: draftText,
                    })
                  }
                >
                  Save and check
                </button>
                {updateDocument.error && (
                  <p className="error">{updateDocument.error.message}</p>
                )}
              </div>
            ) : (
              <form className="editor-placeholder" onSubmit={submitDocument}>
                <p className="eyebrow">New document</p>
                <h1>Start a scene</h1>
                <label>
                  Title
                  <input name="title" required />
                </label>
                <label>
                  Kind
                  <select name="kind" defaultValue="scene">
                    <option value="scene">Scene</option>
                    <option value="chapter">Chapter</option>
                    <option value="manuscript">Manuscript</option>
                    <option value="wiki">Wiki</option>
                    <option value="note">Note</option>
                  </select>
                </label>
                <label>
                  Text
                  <textarea name="text" rows={8} />
                </label>
                <button type="submit" disabled={createDocument.isPending}>
                  Create
                </button>
              </form>
            )}
          </section>
        ) : (
          <>
            <div className="tabs">
              <button type="button" className="active">
                Overview
              </button>
              <button type="button" onClick={() => setView("write")}>
                Write
              </button>
              <button
                type="button"
                onClick={() => {
                  setWorldLens("wall");
                  setView("world");
                }}
              >
                World
              </button>
            </div>
            <section className="editor-placeholder">
              <p className="eyebrow">{selectedProject.kind}</p>
              <h1>{selectedProject.name}</h1>
              <p>
                Pin scraps on the wall, sketch a rough map, or start a scene.
                None of those is the gate. Confirmed facts still live in the
                inspector; the engine only reports after you settle them.
              </p>
              <h2>Documents</h2>
              {documents.isPending && <p>Loading…</p>}
              {documents.data?.length === 0 && <p>No documents yet.</p>}
              <ul>
                {documents.data?.map((document) => (
                  <li key={document.id}>
                    <button
                      type="button"
                      className="linkish"
                      onClick={() => selectDocument(document)}
                    >
                      {document.title}
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          </>
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
                  {createFact.error && (
                    <p className="error">{createFact.error.message}</p>
                  )}
                </form>
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
        {!diagnostics.data?.length && (
          <p className="status">No diagnostics.</p>
        )}
        <ul className="problem-list">
          {diagnostics.data?.map((item) => (
            <li key={item.id} className={item.severity}>
              <code>{item.code}</code>
              <span>{item.message}</span>
            </li>
          ))}
        </ul>
      </section>

      <footer className="statusbar">
        <span>Local first</span>
        <span>{diagnostics.data?.length ?? 0} diagnostics</span>
        <span>Margin 0.1.0</span>
      </footer>
    </div>
  );
}

export default App;
