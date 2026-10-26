import { useEffect, useRef } from "react";
import { Icon } from "../icons";
import type { Preview } from "./bridge";

/**
 * "What will be sent": the exact request the shell would make, with the key
 * shown as a placeholder. It is built by the same code that sends the request,
 * and its size and SHA-256 are the size and digest of the bytes that go out.
 */
export function PreviewDialog({
  preview,
  title,
  onClose,
  onSend,
  sendLabel = "Send",
}: {
  preview: Preview;
  title?: string;
  onClose: () => void;
  onSend?: () => void;
  sendLabel?: string;
}) {
  const closeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    closeRef.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="wb-modal-backdrop" onMouseDown={onClose}>
      <div
        className="wb-dialog preview-dialog"
        role="dialog"
        aria-modal="true"
        aria-label={title ?? "What will be sent"}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="preview-head">
          <h2>{title ?? "What will be sent"}</h2>
          <button
            ref={closeRef}
            type="button"
            className="wb-icon-btn"
            aria-label="Close"
            onClick={onClose}
          >
            <Icon name="x" />
          </button>
        </header>
        {preview.refused ? (
          <p>
            This asks for prose, so nothing would be sent to any provider. I
            never write, rewrite or continue your prose.
          </p>
        ) : (
          <>
            <dl className="preview-facts">
              <dt>Goes to</dt>
              <dd>
                <strong>{preview.host}</strong> · {preview.provider_name} ·{" "}
                <code>{preview.model}</code>{" "}
                {preview.local && (
                  <span className="wb-pill wb-pill--ok">
                    this computer only
                  </span>
                )}
              </dd>
              <dt>Request</dt>
              <dd>
                <code>POST {preview.url}</code>
              </dd>
              <dt>Headers</dt>
              <dd>
                {preview.headers.map(([name, value]) => (
                  <div key={name}>
                    <code>{name}</code>: <code>{value}</code>
                  </div>
                ))}
              </dd>
              <dt>Size</dt>
              <dd>
                {preview.bytes.toLocaleString("en")} bytes · about{" "}
                {preview.tokens_estimate.toLocaleString("en")} tokens ·{" "}
                <code title="SHA-256 of the bytes that would be sent">
                  sha256 {preview.sha256.slice(0, 16)}…
                </code>
              </dd>
              <dt>Your writing</dt>
              <dd>
                {preview.contains_manuscript_text ? (
                  <>
                    <span className="wb-pill wb-pill--warn">
                      a passage you picked
                    </span>{" "}
                    {preview.passages
                      .map((p) => `“${p.label}” (${p.chars} characters)`)
                      .join(", ")}
                  </>
                ) : (
                  <>
                    <span className="wb-pill wb-pill--ok">none</span> No
                    manuscript wording is in this request.
                  </>
                )}
              </dd>
              <dt>Tools offered</dt>
              <dd>
                {preview.json_protocol
                  ? "Described in the prompt (this model has no native tool calling)."
                  : preview.tools_offered.join(", ") || "none"}
              </dd>
            </dl>
            <h3>The request body</h3>
            <pre className="preview-body" tabIndex={0}>
              {preview.body}
            </pre>
            <ul className="preview-notes">
              {preview.notes.map((note) => (
                <li key={note}>{note}</li>
              ))}
            </ul>
          </>
        )}
        <div className="dialog-actions">
          <button type="button" className="wb-btn" onClick={onClose}>
            Close
          </button>
          {onSend && !preview.refused && (
            <button
              type="button"
              className="wb-btn wb-btn--primary"
              onClick={onSend}
            >
              {sendLabel}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
