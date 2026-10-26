import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { Pencil, type Note, type PencilMode } from "./Pencil";
import { leader, loopAround, underline, type Box, type Stroke } from "./strokes";
import { reducedMotion } from "./random";

interface Chapter {
  number: string;
  name: string;
  body: ReactNode;
}

interface Passage {
  id: string;
  code: string;
  chapters: [Chapter, Chapter];
  note: string[];
  /** What the marks say, for people who do not see them. */
  said: string;
}

/** Original sentences for the sample novel, "Salt and Lantern". The engine marks these the way it marks any page. */
const passages: Passage[] = [
  {
    id: "eyes",
    code: "CANON-E013",
    chapters: [
      {
        number: "1",
        name: "The lamp at Gull Point",
        body: (
          <>
            Mara Vell climbed the last ninety steps with the oil can knocking against her knee. Mara’s{" "}
            <span className="pencil-target" data-target="a">grey eyes</span> stung in the wind.
          </>
        ),
      },
      {
        number: "2",
        name: "Night crossing",
        body: (
          <>
            When she turned back, the lamp at Gull Point was out. Mara’s{" "}
            <span className="pencil-target" data-target="b">brown eyes</span> found the headland and found nothing there.
          </>
        ),
      },
    ],
    note: ["ch. 1 says", "grey. Keep", "which?"],
    said: "Chapter 2 gives Mara brown eyes. Chapter 1 settled them as grey. The pencil circles “brown eyes” and asks which to keep.",
  },
  {
    id: "wen",
    code: "CANON-E101",
    chapters: [
      {
        number: "1",
        name: "The lamp at Gull Point",
        body: (
          <>
            Old Wen <span className="pencil-target" data-target="a">had died in the spring</span>; the lamp room still smelled of his pipe and old rope.
          </>
        ),
      },
      {
        number: "3",
        name: "What the water kept",
        body: (
          <>
            <span className="pencil-target" data-target="b">Old Wen</span> trimmed the wick, hummed to himself, and logged the ship that had not come.
          </>
        ),
      },
    ],
    note: ["Wen is dead", "by ch. 1.", "A memory?"],
    said: "Chapter 3 has Old Wen trimming the wick after chapter 1 said he had died. The pencil circles his name and asks if it is a memory.",
  },
  {
    id: "ida",
    code: "NAME-W001",
    chapters: [
      {
        number: "2",
        name: "Ida’s archive",
        body: (
          <>
            <span className="pencil-target" data-target="a">Ida Fenn</span> kept the town’s papers in a room above the chandler’s, between a barometer and a cat.
          </>
        ),
      },
      {
        number: "3",
        name: "What the water kept",
        body: (
          <>
            <span className="pencil-target" data-target="b">Ada Fenn</span> stood at the archive window and counted the boats that had not come back.
          </>
        ),
      },
    ],
    note: ["Ada isn’t", "in the cast.", "A slip?"],
    said: "Chapter 3 says Ada Fenn where the cast has Ida Fenn. The pencil circles “Ada” and asks if it is a slip.",
  },
];

function offsetBox(el: HTMLElement, root: HTMLElement): Box {
  let x = 0;
  let y = 0;
  let node: HTMLElement | null = el;
  while (node && node !== root) {
    x += node.offsetLeft;
    y += node.offsetTop;
    node = node.offsetParent as HTMLElement | null;
  }
  return { x, y, w: el.offsetWidth, h: el.offsetHeight };
}

interface Geometry {
  w: number;
  h: number;
  a: Box;
  b: Box;
  noteX: number;
  noteSize: number;
}

/**
 * A page of the sample novel, typed on paper, with the red pencil at work on it: a loop round the line that
 * disagrees, a line under the one it disagrees with, a leader, and a note in the margin in a hand. The marks
 * are drawn once as the page arrives, and again, differently, when the page is turned or the pointer returns.
 */
export function Manuscript() {
  const [index, setIndex] = useState(0);
  const [epoch, setEpoch] = useState(0);
  const [mode, setMode] = useState<PencilMode>("idle");
  const [geo, setGeo] = useState<Geometry | null>(null);
  const [turning, setTurning] = useState(false);
  const sheet = useRef<HTMLDivElement>(null);
  const lastStart = useRef(0);
  const pending = useRef<(() => void) | null>(null);
  const still = useRef(false);
  const passage = passages[index];

  const measure = useCallback(() => {
    const root = sheet.current;
    if (!root) return;
    const a = root.querySelector<HTMLElement>('[data-target="a"]');
    const b = root.querySelector<HTMLElement>('[data-target="b"]');
    if (!a || !b) return;
    const margin = Math.min(150, Math.max(96, root.offsetWidth * 0.3));
    setGeo({
      w: root.offsetWidth,
      h: root.offsetHeight,
      a: offsetBox(a, root),
      b: offsetBox(b, root),
      noteX: root.offsetWidth - margin + 4,
      noteSize: Math.min(21, margin / 6.4),
    });
  }, []);

  // Measure after layout and again when the fonts arrive or the width changes.
  useLayoutEffect(() => {
    if (reducedMotion()) {
      still.current = true;
      setMode("still");
    }
    measure();
  }, [measure, index]);
  useEffect(() => {
    const root = sheet.current;
    if (!root) return;
    let live = true;
    void document.fonts?.ready.then(() => {
      if (live) measure();
    });
    const ro = new ResizeObserver(() => {
      if (still.current === false) setMode((m) => (m === "draw" ? "still" : m));
      measure();
    });
    ro.observe(root);
    return () => {
      live = false;
      ro.disconnect();
    };
  }, [measure]);

  // First draw: a beat after the page is up, so the black text lands before the red.
  useEffect(() => {
    if (!geo || mode !== "idle" || lastStart.current !== 0) return;
    const t = window.setTimeout(() => {
      lastStart.current = performance.now();
      setMode("draw");
    }, 650);
    return () => window.clearTimeout(t);
  }, [geo, mode]);

  const redraw = useCallback(
    (swap?: () => void) => {
      if (still.current || mode === "erase") return;
      pending.current = swap ?? null;
      setMode("erase");
    },
    [mode],
  );

  const onErased = useCallback(() => {
    pending.current?.();
    pending.current = null;
    setEpoch((e) => e + 1);
    lastStart.current = performance.now();
    setMode("draw");
  }, []);

  const onEnter = () => {
    // The pointer coming back to the page is a second look: rub out, draw again, but not more than once in a few seconds.
    if (mode === "draw" && performance.now() - lastStart.current > 4200) redraw();
  };

  const turn = () => {
    if (turning) return;
    const next = () => setIndex((i) => (i + 1) % passages.length);
    if (still.current) {
      next();
      setEpoch((e) => e + 1);
      return;
    }
    setTurning(true);
    redraw(() => {
      next();
      setGeo(null);
    });
    window.setTimeout(() => setTurning(false), 520);
  };

  const { strokes, notes } = useMemo(() => {
    if (!geo) return { strokes: [] as Stroke[], notes: [] as Note[] };
    const seed = 100 + index * 17 + epoch * 5;
    const list: Stroke[] = [];
    const underlineY = geo.a.y + geo.a.h - 2;
    list.push(underline("under", geo.a.x, geo.a.x + geo.a.w, underlineY, seed, 0.05));
    const c1 = loopAround("loop1", geo.b, seed + 1, 0.55, 1);
    list.push(c1);
    list.push(loopAround("loop2", geo.b, seed + 2, 0.55 + c1.duration * 0.62, 2));
    const from = { x: geo.b.x + geo.b.w + 14, y: geo.b.y + geo.b.h / 2 };
    const to = { x: geo.noteX - 6, y: geo.b.y + geo.b.h / 2 - 4 };
    const lead = leader("lead", from, to, seed + 3, 0.55 + c1.duration * 0.95);
    list.push(...lead);
    const noteDelay = 0.55 + c1.duration * 0.95 + lead[0].duration + 0.12;
    const noteList: Note[] = [
      { id: "note", x: geo.noteX, y: geo.b.y + geo.b.h / 2 + 1, lines: passage.note, delay: noteDelay, perLine: 0.62, size: geo.noteSize, rotate: -2.5 + ((seed % 5) - 2) * 0.5 },
    ];
    return { strokes: list, notes: noteList };
  }, [geo, index, epoch, passage.note]);

  const paneMode: PencilMode = still.current ? "still" : mode;

  return (
    <div className="manuscript-wrap" onPointerEnter={onEnter}>
      <div className={`manuscript${turning ? " is-turning" : ""}`} ref={sheet}>
        <div className="manuscript-head" aria-hidden="true">
          <span>Salt and Lantern</span>
          <span>41</span>
        </div>
        <div className="manuscript-body">
          {passage.chapters.map((chapter, i) => (
            <section key={`${passage.id}-${i}`}>
              <h3>
                <span>{chapter.number}</span> {chapter.name}
              </h3>
              <p>{chapter.body}</p>
            </section>
          ))}
        </div>
        {geo ? (
          <Pencil className="manuscript-pencil" strokes={strokes} notes={notes} width={geo.w} height={geo.h} mode={paneMode} epoch={epoch + index * 100} onErased={onErased} />
        ) : null}
        <p className="visually-hidden">{passage.said}</p>
        <span className="chip chip--code manuscript-code">{passage.code}</span>
      </div>
      <button type="button" className="manuscript-turn" onClick={turn} aria-label="Show another page of the sample novel">
        another page
      </button>
    </div>
  );
}
