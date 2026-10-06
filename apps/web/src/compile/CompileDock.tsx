import { useState } from "react";
import type { CompileArtifact, CompileFormat } from "@asten/contracts";

interface Props {
  compiling?: boolean;
  artifact?: CompileArtifact;
  onCompile: (format: CompileFormat) => void;
  onImportFile: (file: File) => void;
}

const FORMATS: { id: CompileFormat; label: string }[] = [
  { id: "markdown", label: "Markdown" },
  { id: "plain", label: "Plain text" },
  { id: "fountain", label: "Fountain" },
  { id: "html", label: "HTML" },
  { id: "docx", label: "Word (.docx)" },
];

export function CompileDock({
  compiling,
  artifact,
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
    <section className="editor-placeholder">
      <p className="eyebrow">Compile</p>
      <h1>Take it with you</h1>
      <p>
        The binder becomes a manuscript. Source is not mutated. Word and Docs
        remain guests.
      </p>
      <div className="actions-row">
        {FORMATS.map((item) => (
          <button
            key={item.id}
            type="button"
            className={format === item.id ? "active" : ""}
            onClick={() => setFormat(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>
      <button
        type="button"
        disabled={compiling}
        onClick={() => onCompile(format)}
      >
        {compiling ? "Compiling…" : `Compile ${format}`}
      </button>
      {artifact && (
        <p>
          Ready: {artifact.filename}{" "}
          <button type="button" className="linkish" onClick={download}>
            Download
          </button>
        </p>
      )}
      <h2>Bring prose home</h2>
      <input
        type="file"
        accept=".md,.txt,.fountain,.html,.docx"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) onImportFile(file);
        }}
      />
    </section>
  );
}
