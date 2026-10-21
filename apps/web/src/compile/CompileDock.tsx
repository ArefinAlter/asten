import { useState } from "react";
import type { CompileArtifact, CompileFormat } from "@asten/contracts";
import { Glyph } from "../brand/Glyphs";

interface Props {
  compiling?: boolean;
  artifact?: CompileArtifact;
  error?: string;
  onCompile: (format: CompileFormat) => void;
  onImportFile: (file: File) => void;
}

const FORMATS: { id: CompileFormat; label: string }[] = [
  { id: "markdown", label: "Markdown" },
  { id: "plain", label: "Plain text" },
  { id: "fountain", label: "Fountain" },
  { id: "html", label: "HTML (print to PDF)" },
  { id: "docx", label: "Word (.docx)" },
  { id: "epub", label: "EPUB" },
  { id: "fdx", label: "Final Draft (.fdx)" },
];

export function CompileDock({
  compiling,
  artifact,
  error,
  onCompile,
  onImportFile,
}: Props) {
  const [format, setFormat] = useState<CompileFormat>("markdown");

  function download() {
    if (!artifact) return;
    const bytes =
      artifact.encoding === "base64"
        ? Uint8Array.from(atob(artifact.body), (char) => char.charCodeAt(0))
        : new TextEncoder().encode(artifact.body);
    const blob = new Blob([bytes], { type: artifact.mime });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = artifact.filename;
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <section className="compile register">
      <header>
        <p className="eyebrow mono">Compile</p>
        <h1>Take it with you</h1>
        <p className="lede">
          The binder becomes a manuscript. Source is not mutated. Word and Docs
          remain guests.
        </p>
      </header>
      <div className="compile-columns">
        <div>
          <h2>
            <Glyph name="kit" /> Compile
          </h2>
          <div className="segmented wrap" role="tablist">
            {FORMATS.map((item) => (
              <button
                key={item.id}
                type="button"
                role="tab"
                aria-selected={format === item.id}
                className={format === item.id ? "active" : ""}
                onClick={() => setFormat(item.id)}
              >
                {item.label}
              </button>
            ))}
          </div>
          <button
            type="button"
            className="btn btn--primary"
            disabled={compiling}
            onClick={() => onCompile(format)}
          >
            {compiling ? "Compiling…" : `Compile ${format}`}
          </button>
          {artifact && (
            <p className="hint">
              Ready: {artifact.filename}{" "}
              <button type="button" className="linkish" onClick={download}>
                Download
              </button>
            </p>
          )}
        </div>
        <div>
          <h2>
            <Glyph name="door" /> Bring prose home
          </h2>
          <p className="hint">
            Markdown, plain text, Fountain, Final Draft (.fdx), HTML, or .docx.
            Each top-level paragraph becomes a block with a stable id, so the
            engine can follow it through every later edit.
          </p>
          <input
            type="file"
            accept=".md,.txt,.fountain,.fdx,.html,.htm,.docx"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) onImportFile(file);
            }}
          />
        </div>
      </div>
      {error && <p className="error">{error}</p>}
    </section>
  );
}
