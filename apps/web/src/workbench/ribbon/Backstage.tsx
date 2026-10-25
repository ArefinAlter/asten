import { RELEASE } from "../../version";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import type { CompileFormat, ProjectKind } from "@asten/contracts";
import { PaletteMark } from "../../brand/Palette";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { log } from "../output";
import { downloadArtifact, relativeTime } from "../util";
import { BACKSTAGE_PAGES, type BackstagePage } from "./model";

const FORMATS: { format: CompileFormat; label: string; note: string }[] = [
  {
    format: "docx",
    label: "Word document (.docx)",
    note: "For editors and agents who work in Word.",
  },
  { format: "epub", label: "EPUB 3", note: "An e-book in binder order." },
  {
    format: "markdown",
    label: "Markdown",
    note: "Plain text with headings; good for version control.",
  },
  {
    format: "html",
    label: "HTML",
    note: "One page; print it from the browser.",
  },
  {
    format: "plain",
    label: "Plain text",
    note: "Words only, scene breaks as # # #.",
  },
  { format: "fountain", label: "Fountain", note: "Screenplay plain text." },
  {
    format: "fdx",
    label: "Final Draft (.fdx)",
    note: "Screenplay for Final Draft.",
  },
];

const TEMPLATES: { kind: ProjectKind; label: string; note: string }[] = [
  {
    kind: "novel",
    label: "Novel",
    note: "Chapters and scenes, a cast and a clock.",
  },
  {
    kind: "screenplay",
    label: "Screenplay",
    note: "Scenes in Fountain order; compiles to FDX.",
  },
  {
    kind: "non_fiction",
    label: "Non-fiction",
    note: "Chapters, sources and notes.",
  },
  { kind: "other", label: "Blank", note: "Nothing but a binder." },
];

export function Backstage({
  page,
  onPage,
  onClose,
  onOptions,
  onImport,
  onSelectProject,
  onSample,
}: {
  page: BackstagePage;
  onPage: (page: BackstagePage) => void;
  onClose: () => void;
  onOptions: () => void;
  onImport: (file: File) => void;
  onSelectProject: (id: string) => void;
  /** Create the sample novel as a real project (desktop and local desks). */
  onSample?: () => void;
}) {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const fileInput = useRef<HTMLInputElement>(null);
  const [name, setName] = useState("");
  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => api.client.listProjects(),
  });
  const desk = useQuery({
    queryKey: ["desk-info"],
    queryFn: () => api.client.deskInfo(),
    retry: false,
  });
  const create = useMutation({
    mutationFn: (input: { name: string; kind: ProjectKind }) =>
      api.client.createProject(input),
    onSuccess: async (project) => {
      await queryClient.invalidateQueries({ queryKey: ["projects"] });
      onSelectProject(project.id);
      onClose();
    },
  });
  const compile = useMutation({
    mutationFn: (format: CompileFormat) =>
      api.client.compileProject(api.projectId!, { format }),
    onSuccess: (artifact, format) => {
      log("Compile", `Compiled ${artifact.filename} (${format}).`);
      downloadArtifact(artifact);
    },
    onError: (error, format) =>
      log("Compile", `${format}: ${error.message}`, "error"),
  });
  const words = api.documents.reduce((sum, d) => sum + d.words, 0);
  const open =
    api.diagnostics?.diagnostics.filter((d) => !d.suppressed_by) ?? [];

  return (
    <div
      className="backstage"
      role="dialog"
      aria-modal="true"
      aria-label="File"
    >
      <nav className="backstage-nav" aria-label="File pages">
        <button
          type="button"
          className="backstage-back"
          onClick={onClose}
          aria-label="Back to the page"
        >
          <Icon name="chevronRight" className="flip" /> Back
        </button>
        {BACKSTAGE_PAGES.map((item) => (
          <button
            key={item.id}
            type="button"
            className={item.id === page ? "active" : ""}
            onClick={() =>
              item.id === "options" ? onOptions() : onPage(item.id)
            }
          >
            {item.label}
          </button>
        ))}
      </nav>
      <div className="backstage-body">
        {page === "home" && (
          <section>
            <h1>Good {greeting()}</h1>
            <h2>New</h2>
            <div className="backstage-templates">
              {TEMPLATES.slice(0, 3).map((t) => (
                <button
                  key={t.kind}
                  type="button"
                  className="template"
                  onClick={() => onPage("new")}
                >
                  <span className="template-sheet" />
                  <strong>{t.label}</strong>
                </button>
              ))}
            </div>
            <h2>Recent pages</h2>
            <table className="wb-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Kind</th>
                  <th>Words</th>
                  <th>Changed</th>
                </tr>
              </thead>
              <tbody>
                {[...api.documents]
                  .filter(
                    (d) =>
                      d.kind === "scene" ||
                      d.kind === "wiki" ||
                      d.kind === "note",
                  )
                  .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
                  .slice(0, 8)
                  .map((d) => (
                    <tr
                      key={d.id}
                      className="clickable"
                      onClick={() => {
                        api.openDocument(d.id);
                        onClose();
                      }}
                    >
                      <td>{d.title}</td>
                      <td className="muted">{d.kind}</td>
                      <td className="mono">{d.words}</td>
                      <td className="muted">{relativeTime(d.updated_at)}</td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </section>
        )}
        {page === "new" && (
          <section>
            <h1>New</h1>
            <label className="field">
              <span>Project name</span>
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Untitled project"
              />
            </label>
            <div className="backstage-templates">
              {TEMPLATES.map((t) => (
                <button
                  key={t.kind}
                  type="button"
                  className="template"
                  disabled={create.isPending}
                  onClick={() =>
                    create.mutate({
                      name: name.trim() || `Untitled ${t.label.toLowerCase()}`,
                      kind: t.kind,
                    })
                  }
                >
                  <span className="template-sheet" />
                  <strong>{t.label}</strong>
                  <small>{t.note}</small>
                </button>
              ))}
              {onSample && (
                <button type="button" className="template" onClick={onSample}>
                  <span className="template-sheet" />
                  <strong>Sample novel</strong>
                  <small>
                    “Salt and Lantern”, a short story written for Asten, for
                    trying the desk.
                  </small>
                </button>
              )}
            </div>
            {create.error && <p className="wb-error">{create.error.message}</p>}
          </section>
        )}
        {page === "open" && (
          <section>
            <h1>Open</h1>
            <div className="wb-card wb-rows">
              {(projects.data ?? []).map((project) => (
                <button
                  key={project.id}
                  type="button"
                  className="wb-row clickable"
                  onClick={() => {
                    onSelectProject(project.id);
                    onClose();
                  }}
                >
                  <Icon name="book" />
                  <span className="wb-row-main">
                    <strong>{project.name}</strong>
                    <small>{project.kind.replace("_", " ")}</small>
                  </span>
                  <span className="muted">
                    {relativeTime(project.updated_at)}
                  </span>
                </button>
              ))}
            </div>
            <h2>Import a file</h2>
            <p className="muted">
              Markdown, plain text, HTML, Fountain, Word (.docx) or Final Draft
              (.fdx) becomes a new page in the binder.
            </p>
            <input
              ref={fileInput}
              type="file"
              hidden
              accept=".md,.txt,.html,.htm,.fountain,.docx,.fdx"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) onImport(file);
                e.target.value = "";
              }}
            />
            <button
              type="button"
              className="wb-btn"
              disabled={!api.projectId}
              onClick={() => fileInput.current?.click()}
            >
              <Icon name="upload" /> Choose a file…
            </button>
          </section>
        )}
        {page === "info" && (
          <section>
            <h1>Info</h1>
            {api.project ? (
              <div className="backstage-info">
                <div className="wb-card wb-rows">
                  <Row label="Project" value={api.project.name} />
                  <Row
                    label="Kind"
                    value={api.project.kind.replace("_", " ")}
                  />
                  <Row label="Pages" value={String(api.documents.length)} />
                  <Row label="Words" value={words.toLocaleString("en")} />
                  <Row label="Open marks" value={String(open.length)} />
                  <Row
                    label="Revision"
                    value={api.diagnostics?.revision.slice(0, 12) ?? "…"}
                    mono
                  />
                  <Row
                    label="Created"
                    value={new Date(api.project.created_at).toLocaleString(
                      "en-GB",
                    )}
                  />
                  <Row
                    label="Lives in"
                    value={
                      desk.data?.database_path ??
                      (api.sample
                        ? "This page's memory (sample data)"
                        : "The API's database")
                    }
                    mono
                  />
                </div>
                <div className="wb-card">
                  <h3>Protect</h3>
                  <p className="muted">
                    Every page keeps a hash-chained history of its edits. Verify
                    it, see who wrote each line, or bisect a mark to the edit
                    that caused it.
                  </p>
                  <button
                    type="button"
                    className="wb-btn"
                    onClick={() => {
                      api.setView("history");
                      onClose();
                    }}
                  >
                    Open History
                  </button>
                </div>
              </div>
            ) : (
              <p className="muted">Open a project to see its details.</p>
            )}
          </section>
        )}
        {page === "saveAs" && (
          <section>
            <h1>Save as</h1>
            <p>
              Asten saves as you write: every pause commits the page to the
              project file. There is no Save button to forget.
            </p>
            <div className="wb-card wb-rows">
              <button
                type="button"
                className="wb-row clickable"
                disabled={!api.openDocumentId}
                onClick={() => {
                  api.run("app.save");
                  onClose();
                }}
              >
                <Icon name="commit" />
                <span className="wb-row-main">
                  <strong>Save a named version</strong>
                  <small>
                    A snapshot of the open page, listed in Source Control.
                  </small>
                </span>
              </button>
              <button
                type="button"
                className="wb-row clickable"
                onClick={() => onPage("export")}
              >
                <Icon name="download" />
                <span className="wb-row-main">
                  <strong>Save a copy in another format</strong>
                  <small>Compile the manuscript to a file.</small>
                </span>
              </button>
            </div>
          </section>
        )}
        {page === "export" && (
          <section>
            <h1>Export / Compile</h1>
            <p className="muted">
              Compile writes the manuscript in binder order, leaving out pages
              marked “not in compile”.
            </p>
            <div className="format-grid">
              {FORMATS.map((f) => (
                <button
                  key={f.format}
                  type="button"
                  className="format-card"
                  disabled={!api.projectId || compile.isPending}
                  onClick={() => compile.mutate(f.format)}
                >
                  <strong>{f.label}</strong>
                  <small>{f.note}</small>
                </button>
              ))}
            </div>
            {compile.isPending && <p className="muted">Compiling…</p>}
            {compile.error && (
              <p className="wb-error">{compile.error.message}</p>
            )}
          </section>
        )}
        {page === "print" && (
          <section>
            <h1>Print</h1>
            <p>
              Printing uses your system's print dialog on the page as it is laid
              out (Layout sets the size and margins). The ribbon, panels and
              marks are left out.
            </p>
            <button
              type="button"
              className="wb-btn wb-btn--primary"
              disabled={!api.openDocumentId}
              onClick={() => {
                onClose();
                window.setTimeout(() => window.print(), 50);
              }}
            >
              <Icon name="page" /> Print the open page
            </button>
            {!api.openDocumentId && <p className="muted">Open a page first.</p>}
          </section>
        )}
        {page === "share" && (
          <section>
            <h1>Share</h1>
            <p>
              Asten has no cloud and no accounts: your manuscript never leaves
              this machine unless you send a file. Two ways to work with others:
            </p>
            <div className="wb-card wb-rows">
              <button
                type="button"
                className="wb-row clickable"
                onClick={() => onPage("export")}
              >
                <Icon name="download" />
                <span className="wb-row-main">
                  <strong>Send a compiled file</strong>
                  <small>Word, EPUB, Markdown and more.</small>
                </span>
              </button>
              <button
                type="button"
                className="wb-row clickable"
                onClick={() => {
                  api.openSettings("guests");
                  onClose();
                }}
              >
                <Icon name="plug" />
                <span className="wb-row-main">
                  <strong>Pair Word or Google Docs</strong>
                  <small>
                    The add-ins read the Scales beside a document you open
                    there.
                  </small>
                </span>
              </button>
            </div>
          </section>
        )}
        {page === "account" && (
          <section>
            <h1>Account</h1>
            <div className="wb-card account-card">
              <PaletteMark />
              <div>
                <strong>No account</strong>
                <p className="muted">
                  Asten has no accounts, no sync and no telemetry. There is
                  nothing to sign in to.
                </p>
              </div>
            </div>
            <div className="wb-card wb-rows">
              <Row label="Release" value={`Pre-alpha release ${RELEASE}`} />
              <Row
                label="Data"
                value={
                  api.sample
                    ? "Sample project, in this page's memory"
                    : "Your projects, on this machine"
                }
              />
              <Row label="Made by" value="Verne Ltd." />
            </div>
          </section>
        )}
      </div>
    </div>
  );
}

function Row({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="wb-row">
      <span className="wb-row-label">{label}</span>
      <span className={`wb-row-value${mono ? " mono" : ""}`}>{value}</span>
    </div>
  );
}

function greeting() {
  const hour = new Date().getHours();
  return hour < 12 ? "morning" : hour < 18 ? "afternoon" : "evening";
}
