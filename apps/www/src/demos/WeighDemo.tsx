import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { Pencil, type PencilMode } from "../personality/Pencil";
import { loopAround, type Box, type Stroke } from "../personality/strokes";
import { reducedMotion, springStep } from "../personality/random";

/*
 * The balance. Two pans hang from a beam: your page on the left, your canon on the right. When the page says
 * something the canon does not, the beam tips under the page; when a fix makes them agree it comes level. The
 * beam is a damped spring (it overshoots once, by a few degrees, and settles), the pans sway on their own small
 * springs, and nothing runs once the beam is still. The examples are real: the wording of each mark and of its
 * repairs is what asten-api returns for the same facts.
 */

interface Option {
  value: string;
  agrees: boolean;
}

interface Fix {
  id: string;
  label: string;
  /** Does applying it make the page and the canon agree? */
  clears: boolean;
  /** What the engine says or does. */
  result: string;
  /** For "Retire": the canon takes on the page's value. */
  adopts?: boolean;
}

interface Case {
  id: string;
  name: string;
  /** The settled fact, as the canon card shows it. */
  fact: (adopted: string | null) => { entity: string; predicate: string; value: string; where: string };
  /** The sentence on the page, with the variable word in the middle. */
  sentence: (word: ReactNode) => ReactNode;
  wordLabel: string;
  options: Option[];
  code: string;
  title: (canonValue: string) => string;
  lines: (word: string, canonValue: string) => string[];
  slip: string;
  fixes: (canonValue: string, word: string) => Fix[];
  agreeing: string;
}

const KEEP = "Kept as written.";
const SILENCED = "Silenced. If the evidence changes, the mark comes back for review.";

const cases: Case[] = [
  {
    id: "hair",
    name: "A hair color",
    fact: (adopted) => ({ entity: "Mira", predicate: "hair", value: adopted ?? "auburn", where: "settled in chapter 2" }),
    sentence: (word) => <>Mira pushed her {word} hair out of her eyes and listened for the bell.</>,
    wordLabel: "Mira’s hair, on the page in chapter 19",
    options: [
      { value: "auburn", agrees: true },
      { value: "black", agrees: false },
      { value: "silver", agrees: false },
    ],
    code: "CANON-E013",
    title: (c) => `Mira’s hair was settled as ${c}.`,
    lines: (w, c) => [`① Settled: Mira · hair · ${c}`, `② Chapter 19 says: “${w}”`, "③ No event between an unplaced time and day 40 changes Mira’s hair."],
    slip: "Most likely slip: ② (93%)",
    fixes: (c, w) => [
      { id: "event", label: "Add an event", clears: true, result: `Added an event: Mira’s hair changes to ${w}. The page and the record agree.` },
      { id: "retire", label: `Retire “${c}”`, clears: true, adopts: true, result: `Retired “${c}” and settled ${w}. The page and the record agree.` },
      { id: "silence", label: "Silence this mark", clears: false, result: SILENCED },
      { id: "keep", label: "Keep my sentence", clears: false, result: KEEP },
    ],
    agreeing: "The page agrees with the canon. The scales are level.",
  },
  {
    id: "death",
    name: "A death",
    fact: () => ({ entity: "Captain Oren", predicate: "status", value: "dead", where: "from day 9" }),
    sentence: (word) => <>On {word}, Captain Oren poured the tea and said nothing.</>,
    wordLabel: "The day chapter 14 is placed on",
    options: [
      { value: "day 3", agrees: true },
      { value: "day 14", agrees: false },
    ],
    code: "CANON-E101",
    title: () => "Captain Oren acts after a settled death.",
    lines: (w) => ["① Captain Oren dies (day 9)", "② Chapter 14 mentions Captain Oren in narration: “Captain Oren”", `③ Chapter 14: placed at ${w}`],
    slip: "Most likely slip: ② (68%)",
    fixes: () => [
      { id: "time", label: "Set the scene’s time", clears: true, result: "Placed chapter 14 on day 3, before the death. The page and the record agree." },
      { id: "move", label: "Move the death later", clears: true, result: "Moved the death to day 15. The page and the record agree." },
      { id: "silence", label: "Silence this mark", clears: false, result: SILENCED },
      { id: "keep", label: "Keep my sentence", clears: false, result: KEEP },
    ],
    agreeing: "The scene sits before the death in story time. The scales are level.",
  },
  {
    id: "journey",
    name: "A journey",
    fact: () => ({ entity: "Veld to the capital", predicate: "route", value: "5 days by sea", where: "declared route" }),
    sentence: (word) => <>Mira left Veld on day 10 and walked the quay in the capital on day {word}.</>,
    wordLabel: "The day Mira reaches the capital",
    options: [
      { value: "12", agrees: false },
      { value: "15", agrees: true },
    ],
    code: "CANON-E301",
    title: () => "Mira cannot reach the capital from Veld in time.",
    lines: (w) => ["① Settled: Mira · location · Veld, day 10", `② Settled: Mira · location · the capital, day ${w}`, "③ The shortest route from Veld to the capital takes 5 days."],
    slip: "Most likely slip: ③ (85%)",
    fixes: () => [
      { id: "route", label: "Add a faster route", clears: true, result: "Added a two-day route from Veld to the capital. The page and the record agree." },
      { id: "silence", label: "Silence this mark", clears: false, result: SILENCED },
      { id: "keep", label: "Keep my sentence", clears: false, result: KEEP },
    ],
    agreeing: "The journey fits the time between the scenes. The scales are level.",
  },
];

const TILT = -9; // degrees: counter-clockwise, so the page pan goes down
const HALF = 150; // half the beam, in svg units

function ends(a: number) {
  const rad = (a * Math.PI) / 180;
  return { lx: -HALF * Math.cos(rad), ly: -HALF * Math.sin(rad), rx: HALF * Math.cos(rad), ry: HALF * Math.sin(rad) };
}

export function WeighDemo({ compact = false }: { compact?: boolean }) {
  const [ci, setCi] = useState(0);
  const [oi, setOi] = useState(1);
  const [fix, setFix] = useState<Fix | null>(null);
  const [outcome, setOutcome] = useState<"silenced" | "kept" | null>(null);
  const c = cases[ci];
  const option = c.options[oi] ?? c.options[0];
  const adopted = fix?.adopts ? option.value : null;
  const fact = c.fact(adopted);
  const canonValue = fact.value;
  const disagrees = !option.agrees && !fix?.clears;
  const marked = disagrees && outcome !== "silenced";

  const pick = (next: number) => {
    setOi(next);
    setFix(null);
    setOutcome(null);
  };
  const choose = (next: number) => {
    setCi(next);
    // Each example opens on the slip, so the beam has something to do.
    setOi(cases[next].options.findIndex((o) => !o.agrees));
    setFix(null);
    setOutcome(null);
  };
  const apply = (f: Fix) => {
    if (f.clears) {
      setFix(f);
      setOutcome(null);
    } else {
      setOutcome(f.id === "silence" ? "silenced" : "kept");
    }
  };

  const status = fix?.result ?? (outcome === "silenced" ? SILENCED : outcome === "kept" ? KEEP : disagrees ? c.title(canonValue) : c.agreeing);
  const fixes = c.fixes(canonValue, option.value);

  // The beam and the pans: a damped spring, stepped by hand, running only while something moves.
  const beam = useRef<SVGGElement>(null);
  const panL = useRef<SVGGElement>(null);
  const panR = useRef<SVGGElement>(null);
  const needle = useRef<SVGLineElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const target = useRef(disagrees ? TILT : 0);
  const state = useRef({ a: disagrees ? TILT : 0, v: 0, sl: 0, sv: 0, sr: 0, svr: 0, raf: 0, last: 0, visible: false, started: false });
  target.current = disagrees ? TILT : 0;

  const paint = useCallback(() => {
    const s = state.current;
    const e = ends(s.a);
    beam.current?.setAttribute("transform", `rotate(${s.a.toFixed(3)})`);
    needle.current?.setAttribute("transform", `rotate(${s.a.toFixed(3)})`);
    panL.current?.setAttribute("transform", `translate(${e.lx.toFixed(2)} ${e.ly.toFixed(2)}) rotate(${s.sl.toFixed(3)})`);
    panR.current?.setAttribute("transform", `translate(${e.rx.toFixed(2)} ${e.ry.toFixed(2)}) rotate(${s.sr.toFixed(3)})`);
  }, []);

  const run = useCallback(() => {
    const s = state.current;
    if (s.raf) return;
    s.last = performance.now();
    const tick = (now: number) => {
      const dt = Math.min(0.05, (now - s.last) / 1000);
      s.last = now;
      const prevV = s.v;
      [s.a, s.v] = springStep(s.a, s.v, target.current, dt, 90, 6.5);
      const acc = (s.v - prevV) / Math.max(dt, 1e-3);
      [s.sl, s.sv] = springStep(s.sl, s.sv + acc * dt * -0.012, 0, dt, 38, 2.4);
      [s.sr, s.svr] = springStep(s.sr, s.svr + acc * dt * -0.012, 0, dt, 38, 2.4);
      paint();
      const resting = Math.abs(s.a - target.current) < 0.01 && Math.abs(s.v) < 0.02 && Math.abs(s.sl) < 0.01 && Math.abs(s.sr) < 0.01;
      if (resting) {
        s.raf = 0;
        s.a = target.current;
        s.sl = s.sr = s.sv = s.svr = 0;
        paint();
        return;
      }
      if (document.hidden) {
        s.raf = 0;
        return;
      }
      s.raf = requestAnimationFrame(tick);
    };
    s.raf = requestAnimationFrame(tick);
  }, [paint]);

  // The server draws the beam already tipped; in a browser it starts level and settles once the section is seen.
  useLayoutEffect(() => {
    if (reducedMotion() || state.current.started) return;
    state.current.a = 0;
    paint();
  }, [paint]);

  useEffect(() => {
    const el = root.current;
    if (!el || reducedMotion()) return;
    const io = new IntersectionObserver(
      ([entry]) => {
        state.current.visible = entry.isIntersecting;
        if (entry.isIntersecting && !state.current.started) {
          state.current.started = true;
          run();
        }
      },
      { threshold: 0.4 },
    );
    io.observe(el);
    const onVis = () => {
      if (!document.hidden && state.current.visible) run();
    };
    document.addEventListener("visibilitychange", onVis);
    return () => {
      io.disconnect();
      document.removeEventListener("visibilitychange", onVis);
      cancelAnimationFrame(state.current.raf);
      state.current.raf = 0;
    };
  }, [run]);

  useEffect(() => {
    if (reducedMotion()) {
      state.current.a = target.current;
      paint();
      return;
    }
    if (state.current.started) run();
  }, [disagrees, run, paint]);

  // The pencil circles the word on the page while it disagrees.
  const sentence = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState<{ w: number; h: number; word: Box } | null>(null);
  const [pencil, setPencil] = useState<PencilMode>("idle");
  const [epoch, setEpoch] = useState(0);
  const measure = useCallback(() => {
    const host = sentence.current;
    const word = host?.querySelector<HTMLElement>("[data-word]");
    if (!host || !word) return;
    let x = 0;
    let y = 0;
    let n: HTMLElement | null = word;
    while (n && n !== host) {
      x += n.offsetLeft;
      y += n.offsetTop;
      n = n.offsetParent as HTMLElement | null;
    }
    setBox({ w: host.offsetWidth, h: host.offsetHeight, word: { x, y, w: word.offsetWidth, h: word.offsetHeight } });
  }, []);
  useLayoutEffect(measure, [measure, ci, oi]);
  useEffect(() => {
    const host = sentence.current;
    if (!host) return;
    const ro = new ResizeObserver(measure);
    ro.observe(host);
    void document.fonts?.ready.then(measure);
    return () => ro.disconnect();
  }, [measure]);
  const wasMarked = useRef(false);
  useEffect(() => {
    if (marked) {
      setEpoch((e) => e + 1);
      setPencil(reducedMotion() ? "still" : "draw");
    } else if (wasMarked.current) {
      setPencil(reducedMotion() ? "idle" : "erase");
    }
    wasMarked.current = marked;
  }, [marked, ci, oi]);
  const strokes = useMemo<Stroke[]>(
    () => (box ? [loopAround("word", box.word, 40 + ci * 9 + oi * 3 + epoch, 0.05, 1), loopAround("word2", box.word, 41 + ci * 9 + oi * 3 + epoch, 0.35, 2)] : []),
    [box, ci, oi, epoch],
  );

  const word = (
    <span className={`balance-word${marked ? " is-marked" : ""}`} data-word>
      {option.value}
    </span>
  );
  const e0 = ends(disagrees ? TILT : 0);

  return (
    <div className={`balance${compact ? " balance--compact" : ""}`} ref={root}>
      <div className="balance-cases" role="radiogroup" aria-label="An example from the sample novel">
        {cases.map((k, i) => (
          <button key={k.id} type="button" role="radio" aria-checked={ci === i} className={ci === i ? "is-on" : undefined} onClick={() => choose(i)}>
            {k.name}
          </button>
        ))}
      </div>

      <div className="balance-stage">
        <div className="balance-card balance-page">
          <p className="balance-tag">Your page</p>
          <div className="balance-sentence" ref={sentence}>
            <p>{c.sentence(word)}</p>
            {box ? <Pencil className="balance-pencil" strokes={strokes} width={box.w} height={box.h} mode={pencil} epoch={epoch} onErased={() => setPencil("idle")} /> : null}
          </div>
          <div className="balance-choices" role="radiogroup" aria-label={c.wordLabel}>
            {c.options.map((o, i) => (
              <button key={o.value} type="button" role="radio" aria-checked={oi === i} className={oi === i ? "is-on" : undefined} onClick={() => pick(i)}>
                {o.value}
              </button>
            ))}
          </div>
        </div>

        <svg className="balance-svg" viewBox="0 0 420 350" role="img" aria-label={disagrees ? "A balance tipped under the page" : "A balance, level"}>
          <ellipse cx="210" cy="332" rx="118" ry="9" className="balance-shadow" />
          <path d="M118 316 h184 q10 0 10 10 v6 h-204 v-6 q0 -10 10 -10z" className="balance-base" />
          <rect x="203" y="82" width="14" height="236" rx="3" className="balance-post" />
          <g transform="translate(210 80)">
            {[-12, -6, 0, 6, 12].map((t) => (
              <line
                key={t}
                x1={Math.sin((t * Math.PI) / 180) * 64}
                y1={Math.cos((t * Math.PI) / 180) * 64}
                x2={Math.sin((t * Math.PI) / 180) * 72}
                y2={Math.cos((t * Math.PI) / 180) * 72}
                className="balance-tick"
              />
            ))}
            <line ref={needle} x1="0" y1="0" x2="0" y2="62" className="balance-needle" transform={`rotate(${disagrees ? TILT : 0})`} />
            <g ref={beam} transform={`rotate(${disagrees ? TILT : 0})`}>
              <path d="M-158 3 Q-158 -4 -150 -4 L-8 -6 Q0 -9 8 -6 L150 -4 Q158 -4 158 3 Q158 8 150 8 L8 6 Q0 9 -8 6 L-150 8 Q-158 8 -158 3z" className="balance-beam" />
              <circle cx="0" cy="2" r="7" className="balance-pivot" />
            </g>
            <g ref={panL} transform={`translate(${e0.lx.toFixed(2)} ${e0.ly.toFixed(2)})`}>
              <Pan side="page" />
            </g>
            <g ref={panR} transform={`translate(${e0.rx.toFixed(2)} ${e0.ry.toFixed(2)})`}>
              <Pan side="canon" />
            </g>
          </g>
        </svg>

        <div className="balance-card balance-canon">
          <p className="balance-tag">Your canon</p>
          <p className="balance-fact">
            <strong>{fact.entity}</strong> · {fact.predicate} · <span className="chip chip--settled">{fact.value}</span>
          </p>
          <p className="balance-where">{fact.where}</p>
        </div>
      </div>

      <div className={`balance-mark${marked ? " is-open" : ""}`}>
        {marked ? (
          <>
            <div className="diagnostic-head">
              <span className="chip chip--code">{c.code}</span>
              <span className="diagnostic-title">{c.title(canonValue)}</span>
            </div>
            <ul className="balance-lines">
              {c.lines(option.value, canonValue).map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
            <p className="balance-slip">{c.slip}</p>
            <div className="balance-fixes">
              {fixes.map((f) => (
                <button key={f.id} type="button" onClick={() => apply(f)}>
                  {f.label}
                </button>
              ))}
            </div>
          </>
        ) : (
          <>
            <p className="balance-status">{status}</p>
            <button type="button" className="balance-again" onClick={() => choose(ci)}>
              Start again
            </button>
          </>
        )}
      </div>
      <p className="visually-hidden" aria-live="polite">
        {status}
      </p>
    </div>
  );
}

/** A pan hanging from a chain: a shallow brass dish, with the page's slip or the feather of Ma'at in it. */
function Pan({ side }: { side: "page" | "canon" }) {
  return (
    <g className="balance-pan">
      <path d="M0 2 L-46 118 M0 2 L46 118 M0 2 L0 108" className="balance-chain" />
      <path d="M-62 118 Q-60 140 0 142 Q60 140 62 118 Z" className="balance-dish" />
      <path d="M-62 118 H62" className="balance-rim" />
      {side === "page" ? (
        <g transform="translate(-24 90) rotate(-3)">
          <rect width="48" height="30" rx="2" className="balance-slip-paper" />
          <path d="M7 9 H41 M7 15 H35 M7 21 H39" className="balance-slip-lines" />
        </g>
      ) : (
        <g transform="translate(0 116)">
          <path d="M-30 0 Q-6 -34 30 -26 Q8 -10 -30 0Z" className="balance-feather" />
          <path d="M-32 1 Q0 -12 28 -25" className="balance-quill" />
        </g>
      )}
    </g>
  );
}
