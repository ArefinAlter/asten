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
 * time below. Where a scene steps backward in story time from the scene
 * before it, CANON-E201 marks the pair, unless you silence it as a flashback.
 */
export function ClocksDemo() {
  const [scenes, setScenes] = useState(initial);
  const [silenced, setSilenced] = useState<Set<string>>(() => new Set());

  const marks = scenes
    .map((scene, i) => ({ scene, prev: scenes[i - 1] }))
    .filter(({ scene, prev }) => prev && scene.day < prev.day)
    .map(({ scene, prev }) => ({ id: scene.id, prev: prev!, scene }));
  const live = marks.filter((mark) => !silenced.has(mark.id));

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
        <p className="clocks-label">Reading order</p>
        <ol>
          {scenes.map((scene, i) => {
            const marked = live.some((mark) => mark.id === scene.id);
            const quiet = silenced.has(scene.id) && marks.some((mark) => mark.id === scene.id);
            return (
              <li key={scene.id} className={`clock-card${marked ? " is-marked" : ""}${quiet ? " is-quiet" : ""}`}>
                <span className="clock-num">Ch. {i + 8}</span>
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
            {marks.length > 0
              ? "Every backward step is a flashback you meant. The clocks are quiet."
              : "Story time only moves forward. Nothing to mark."}
          </p>
        ) : (
          live.map((mark) => (
            <div key={mark.id} className="clocks-mark">
              <span className="chip chip--code">CANON-E201</span>
              <span>
                Reading order places “{mark.prev.title}” (day {mark.prev.day}) before “{mark.scene.title}” (day {mark.scene.day}), but story time runs the other way.
              </span>
              <button type="button" onClick={() => setSilenced((s) => new Set(s).add(mark.id))}>
                It’s a flashback: silence
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
