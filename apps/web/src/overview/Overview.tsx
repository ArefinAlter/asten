import type {
  CanonEntity,
  DiagnosticsSnapshot,
  DocumentSummary,
  Project,
} from "@asten/contracts";
import { Glyph } from "../brand/Glyphs";
import type { View } from "../App";

interface Props {
  project: Project;
  documents: DocumentSummary[];
  entities: CanonEntity[];
  snapshot?: DiagnosticsSnapshot;
  onGo: (view: View) => void;
  onOpenDocument: (document: DocumentSummary) => void;
}

/** The project's front page: what is on the desk and how it weighs. */
export function Overview({
  project,
  documents,
  entities,
  snapshot,
  onGo,
  onOpenDocument,
}: Props) {
  const prose = documents.filter(
    (document) =>
      document.kind === "manuscript" ||
      document.kind === "chapter" ||
      document.kind === "scene",
  );
  // Counted on the server by the same rule the editor applies to the open
  // page, so the binder never needs a body to weigh the project.
  const words = prose.reduce((total, document) => total + document.words, 0);
  const marks =
    snapshot?.diagnostics.filter((mark) => !mark.suppressed_by) ?? [];
  const errors = marks.filter((mark) => mark.severity === "error").length;
  const warnings = marks.filter((mark) => mark.severity === "warning").length;
  const notes = marks.length - errors - warnings;
  const recent = [...documents]
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
    .slice(0, 5);

  return (
    <section className="overview register">
      <header className="overview-head">
        <p className="eyebrow mono">{project.kind.replace("_", " ")}</p>
        <h1>{project.name}</h1>
        <p className="lede">
          Write, pin a scrap, or name a character. None of those is required
          first. Asten weighs each page against what you settle and marks what
          does not balance.
        </p>
      </header>
      <div className="stat-row">
        <Stat
          glyph="scroll"
          label="words"
          value={words.toLocaleString()}
          onClick={() => onGo("write")}
        />
        <Stat
          glyph="seated"
          label="in the cast"
          value={String(entities.length)}
          onClick={() => onGo("cast")}
        />
        <Stat
          glyph="feather"
          label="scenes placed"
          value={`${prose.filter((document) => document.scene.story_time || document.scene.tick != null).length}/${prose.length}`}
          onClick={() => onGo("time")}
        />
        <Stat
          glyph="scales"
          label={errors + warnings === 0 ? "balanced" : "to weigh"}
          value={errors + warnings === 0 ? "✓" : `${errors} · ${warnings}`}
          tone={errors > 0 ? "rubric" : warnings > 0 ? "warn" : "settled"}
          onClick={() => onGo("canon")}
        />
      </div>
      <div className="overview-columns">
        <div>
          <h2>Recently touched</h2>
          <ul className="plain-list">
            {recent.map((document) => (
              <li key={document.id}>
                <button
                  type="button"
                  className="linkish"
                  onClick={() => onOpenDocument(document)}
                >
                  {document.title}
                </button>
                <small className="mono">
                  {document.kind}
                  {document.scene.story_time
                    ? ` · ${document.scene.story_time}`
                    : ""}{" "}
                  · head {document.head_seq}
                </small>
              </li>
            ))}
            {recent.length === 0 && (
              <li className="hint">Nothing yet. The binder is on the left.</li>
            )}
          </ul>
        </div>
        <div>
          <h2>The engine</h2>
          {snapshot ? (
            <ul className="plain-list">
              <li>
                <span>Marks</span>
                <small className="mono">
                  {errors} errors · {warnings} warnings · {notes} notes
                </small>
              </li>
              <li>
                <span>Scripts</span>
                <small className="mono">
                  {snapshot.tiers.length === 0
                    ? "none yet"
                    : snapshot.tiers
                        .map((tier) => `${tier.script} tier ${tier.tier}`)
                        .join(" · ")}
                </small>
              </li>
              <li>
                <span>Last pass</span>
                <small className="mono">
                  {snapshot.timings_ms.total} ms · {snapshot.cached_blocks}/
                  {snapshot.total_blocks} blocks from cache
                </small>
              </li>
              <li>
                <span>Ledger</span>
                <small className="mono">
                  {snapshot.ledger.length} setup
                  {snapshot.ledger.length === 1 ? "" : "s"} awaiting payoff
                </small>
              </li>
            </ul>
          ) : (
            <p className="hint">Weighing…</p>
          )}
          <p className="hint">
            Tier C checks settled facts in any language. Tier B adds names and
            simple attributes read from the page. Models are planned, local, and
            optional.
          </p>
        </div>
      </div>
    </section>
  );
}

function Stat({
  glyph,
  label,
  value,
  tone,
  onClick,
}: {
  glyph: Parameters<typeof Glyph>[0]["name"];
  label: string;
  value: string;
  tone?: string;
  onClick: () => void;
}) {
  return (
    <button type="button" className={`stat ${tone ?? ""}`} onClick={onClick}>
      <Glyph name={glyph} />
      <strong>{value}</strong>
      <span className="mono">{label}</span>
    </button>
  );
}
