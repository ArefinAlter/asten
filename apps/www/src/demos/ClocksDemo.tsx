import { useState } from "react";
import { MoonPhase } from "../art/Night";

interface Scene {
  id: string;
  title: string;
  day: number;
}

const initial: Scene[] = [
  { id: "a", title: "Oren sails", day: 14 },
  { id: "b", title: "The storm", day: 15 },
  { id: "c", title: "The harbor burns", day: 17 },
  { id: "d", title: "Mira remembers the fire", day: 4 },
  { id: "e", title: "Landfall at Veld", day: 18 },
];

const MIN = 1;
const MAX = 21;

/**
 * Two registers, like two bands on a tomb wall: reading order above, story
 * time below. The scenes form one run declared continuous. Inside it, a scene
 * that steps backward in story time from the last scene before it gets a
 * CANON-E201 mark; scenes marked as flashbacks are never compared (D5).
 */
export function ClocksDemo() {
  const [scenes, setScenes] = useState(initial);
  const [flashbacks, setFlashbacks] = useState<Set<string>>(() => new Set(["d"]));

  const live: { id: string; prev: Scene; scene: Scene }[] = [];
  let last: Scene | undefined;
  for (const scene of scenes) {
    if (flashbacks.has(scene.id)) continue;
    if (last && scene.day < last.day) live.push({ id: scene.id, prev: last, scene });
    last = scene;
  }

  function step(id: string, delta: number) {
    setScenes((current) =>
      current.map((scene) =>
        scene.id === id ? { ...scene, day: Math.max(MIN, Math.min(MAX, scene.day + delta)) } : scene,
      ),
    );
  }

  const x = (i: number) => ((i + 0.5) / scenes.length) * 100;
  const t = (day: number) => 4 + ((day - MIN) / (MAX - MIN)) * 92;

  return (
    <div className="clocks">
      <div className="clocks-row clocks-reading">
        <p className="clocks-label">Reading order · run “The siege”, continuous</p>
        <ol>
          {scenes.map((scene, i) => {
            const marked = live.some((mark) => mark.id === scene.id);
            const quiet = flashbacks.has(scene.id);
            return (
              <li key={scene.id} className={`clock-card${marked ? " is-marked" : ""}${quiet ? " is-quiet" : ""}`}>
                <span className="clock-num">Ch. {i + 8}{quiet ? " · flashback" : ""}</span>
                <strong>{scene.title}</strong>
                <span className="clock-day">
                  <button type="button" aria-label={`Move ${scene.title} one day earlier`} onClick={() => step(scene.id, -1)}>
                    −
                  </button>
                  <span>day {scene.day}</span>
                  <button type="button" aria-label={`Move ${scene.title} one day later`} onClick={() => step(scene.id, 1)}>
                    +
                  </button>
                </span>
              </li>
            );
          })}
        </ol>
      </div>

      <svg className="clocks-threads" viewBox="0 0 100 40" preserveAspectRatio="none" aria-hidden="true">
        {scenes.map((scene, i) => {
          const marked = live.some((mark) => mark.id === scene.id);
          return (
            <path
              key={scene.id}
              className={marked ? "thread is-marked" : "thread"}
              d={`M${x(i)} 0 C ${x(i)} 20, ${t(scene.day)} 20, ${t(scene.day)} 40`}
              vectorEffect="non-scaling-stroke"
            />
          );
        })}
      </svg>

      <div className="clocks-row clocks-story">
        <p className="clocks-label">Story time</p>
        <div className="clocks-axis">
          {Array.from({ length: MAX - MIN + 1 }, (_, i) => MIN + i).map((day) => (
            <span key={day} className="clocks-tick" style={{ left: `${t(day)}%` }}>
              {day % 4 === 1 ? <MoonPhase phase={((day - 1) / 28) % 1} r={7} /> : <i />}
            </span>
          ))}
          {scenes.map((scene) => (
            <span key={scene.id} className="clocks-dot" style={{ left: `${t(scene.day)}%` }} />
          ))}
        </div>
      </div>

      <div className="clocks-marks">
        {live.length === 0 ? (
          <p className="clocks-calm">
            {flashbacks.size > 0
              ? "No scene in the run steps backward in story time. Scenes marked as flashbacks are not compared."
              : "Story time only moves forward in this run. Nothing to mark."}
          </p>
        ) : (
          live.map((mark) => (
            <div key={mark.id} className="clocks-mark">
              <span className="chip chip--code">CANON-E201</span>
              <span>
                Inside the continuous run, “{mark.scene.title}” (day {mark.scene.day}) comes after “{mark.prev.title}” (day {mark.prev.day}) in reading order but earlier in story time.
              </span>
              <button type="button" onClick={() => setFlashbacks((s) => new Set(s).add(mark.id))}>
                Mark it a flashback
              </button>
            </div>
          ))
        )}
        <p className="visually-hidden" aria-live="polite">
          {live.length === 0 ? "No clock marks." : `${live.length} clock mark${live.length > 1 ? "s" : ""}: CANON-E201.`}
        </p>
      </div>
    </div>
  );
}
