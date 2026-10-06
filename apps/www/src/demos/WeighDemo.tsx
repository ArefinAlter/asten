import { useEffect, useRef, useState } from "react";
import { Scales } from "../art/Scales";

type Color = "auburn" | "black" | "silver";
type Resolution = null | "kept" | "canon" | "change";

const colors: Color[] = ["auburn", "black", "silver"];

/**
 * Weigh a sentence. The reader picks a word; the page is weighed against
 * canon; if it disagrees, the scales tip and a red mark arrives, a beat after
 * the writer's own ink. The writer decides what happens next.
 */
export function WeighDemo({ compact = false }: { compact?: boolean }) {
  const [color, setColor] = useState<Color>("auburn");
  const [canon, setCanon] = useState<string>("auburn");
  const [resolution, setResolution] = useState<Resolution>(null);
  const [showMark, setShowMark] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  const changeRecorded = resolution === "change";
  const disagrees = !changeRecorded && color !== canon && resolution !== "kept";

  useEffect(() => {
    window.clearTimeout(timer.current);
    if (disagrees) {
      // Black ink first, red ink after a beat.
      timer.current = window.setTimeout(() => setShowMark(true), 380);
    } else {
      setShowMark(false);
    }
    return () => window.clearTimeout(timer.current);
  }, [disagrees, color]);

  function pick(next: Color) {
    setColor(next);
    setResolution(null);
  }

  function reset() {
    setColor("auburn");
    setCanon("auburn");
    setResolution(null);
  }

  const status = disagrees
    ? `Chapter 19 says ${color}, but canon says ${canon}. Asten marks it with CANON-E013.`
    : resolution === "kept"
      ? "You kept your sentence. The mark is silenced, and your reason is stored with the project."
      : resolution === "canon"
        ? `Canon updated: Mira’s hair is now ${canon}. The page and the record agree.`
        : resolution === "change"
          ? `Change recorded: auburn until chapter 12, ${color} after. Both are true in their time.`
          : "The page agrees with canon. The scales are level.";

  const tilt = disagrees ? -11 : 0;

  return (
    <div className={`weigh ${compact ? "weigh--compact" : ""} ${disagrees ? "is-off" : "is-level"}`}>
      <div className="weigh-page">
        <div className="weigh-meta">
          <span className="chip">Chapter 19 · Veld</span>
          <span className="weigh-hint">Change the word ↓</span>
        </div>
        <p className="weigh-prose">
          The tide came in grey over the harbor at Veld. Mira pushed her{" "}
          <span className={`weigh-word ${showMark ? "is-marked" : ""}`}>
            <span className="weigh-choices" role="radiogroup" aria-label="Mira’s hair color in this sentence">
              {colors.map((option) => (
                <button
                  key={option}
                  type="button"
                  role="radio"
                  aria-checked={color === option}
                  className={color === option ? "is-on" : undefined}
                  onClick={() => pick(option)}
                >
                  {option}
                </button>
              ))}
            </span>
            <svg className="weigh-stroke" viewBox="0 0 120 12" preserveAspectRatio="none" aria-hidden="true">
              <path pathLength="1" d="M2 8 C 22 3, 40 10, 60 6 S 98 4, 118 7" />
            </svg>
          </span>{" "}
          hair out of her eyes and listened for the bell.
        </p>
        <div className="weigh-canon">
          <span className="weigh-canon-label">Canon</span>
          <span className="chip chip--settled">
            Mira · hair · {changeRecorded ? `auburn → ${color} (from ch. 12)` : canon}
          </span>
          <span className="weigh-canon-src">settled in chapter 2</span>
        </div>
      </div>

      <div className="weigh-scales">
        <Scales tilt={tilt} marked={disagrees} />
        <p className="weigh-legend">
          <span>your page</span>
          <span>your canon</span>
        </p>
      </div>

      <div className="weigh-mark-slot">
        {showMark ? (
          <div className="weigh-mark">
            <div className="diagnostic-head">
              <span className="chip chip--code">CANON-E013</span>
              <span className="diagnostic-title">Mira’s hair was settled as {canon}.</span>
            </div>
            <p className="weigh-mark-body">
              Chapter 2 says <strong>{canon}</strong>. This page says <strong>{color}</strong>. Which is true is your call.
            </p>
            <div className="weigh-actions">
              <button type="button" onClick={() => setResolution("kept")}>
                Keep my sentence
              </button>
              <button
                type="button"
                onClick={() => {
                  setCanon(color);
                  setResolution("canon");
                }}
              >
                Update canon
              </button>
              <button type="button" onClick={() => setResolution("change")}>
                She dyed it in ch. 12
              </button>
            </div>
          </div>
        ) : (
          <div className={`weigh-calm ${resolution ? "is-resolved" : ""}`}>
            <p>{status}</p>
            {resolution ? (
              <button type="button" className="weigh-reset" onClick={reset}>
                Start again
              </button>
            ) : null}
          </div>
        )}
      </div>
      <p className="visually-hidden" aria-live="polite">
        {status}
      </p>
    </div>
  );
}
