import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import type { CreateProject, ProjectId, ProjectKind } from "@margin/contracts";
import { createMarginClient } from "./lib/margin-client";
import "./margin.css";

const client = createMarginClient();

function App() {
  const [selectedProjectId, setSelectedProjectId] = useState<ProjectId>();
  const [isCreating, setIsCreating] = useState(false);
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
  const createProject = useMutation({
    mutationFn: (input: CreateProject) => client.createProject(input),
    onSuccess: async (project) => {
      setSelectedProjectId(project.id);
      setIsCreating(false);
      await queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });

  function submitProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createProject.mutate({
      name: String(form.get("name") ?? ""),
      kind: String(form.get("kind") ?? "novel") as ProjectKind,
    });
  }

  const selectedProject = projects.data?.find(
    (project) => project.id === selectedProjectId,
  );

  return (
    <div className="app-shell">
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
        {["Write", "Wiki", "Graph", "Time", "Map", "Board"].map((item) => (
          <button type="button" key={item} title={item} disabled>
            {item.slice(0, 1)}
          </button>
        ))}
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
              onClick={() => setSelectedProjectId(project.id)}
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
      </aside>

      <main className="workspace">
        {!selectedProject ? (
          <section className="empty-state">
            <h1>Margin</h1>
            <p>Select a project or create one to begin.</p>
          </section>
        ) : (
          <>
            <div className="tabs">
              <button type="button" className="active">
                Overview
              </button>
            </div>
            <section className="editor-placeholder">
              <p className="eyebrow">{selectedProject.kind}</p>
              <h1>{selectedProject.name}</h1>
              <p>
                The document editor will mount here. The transport, persistence,
                project tree, and diagnostics boundaries are already connected.
              </p>
              <h2>Documents</h2>
              {documents.isPending && <p>Loading…</p>}
              {documents.data?.length === 0 && <p>No documents yet.</p>}
              <ul>
                {documents.data?.map((document) => (
                  <li key={document.id}>{document.title}</li>
                ))}
              </ul>
            </section>
          </>
        )}
      </main>

      <aside className="inspector">
        <div className="pane-heading">Inspector</div>
        <p className="status">Contextual metadata will appear here.</p>
      </aside>

      <section className="problems">
        <div className="pane-heading">Problems</div>
        <p className="status">No diagnostics.</p>
      </section>

      <footer className="statusbar">
        <span>Local first</span>
        <span>0 words</span>
        <span>Margin 0.1.0</span>
      </footer>
    </div>
  );
}

export default App;
