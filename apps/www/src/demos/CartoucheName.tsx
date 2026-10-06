import { useId, useState } from "react";

/**
 * A playful "hieroglyphic alphabet": each Latin letter mapped to a uniliteral
 * sign of similar sound. Real Middle Egyptian spelled differently (and mostly
 * without vowels), and the page says so.
 */
const signs: Record<string, { glyph: string; sign: string }> = {
  a: { glyph: "\u{1313F}", sign: "G1 vulture" },
  b: { glyph: "\u{130C0}", sign: "D58 foot" },
  c: { glyph: "\u{133A1}", sign: "V31 basket" },
  d: { glyph: "\u{130A7}", sign: "D46 hand" },
  e: { glyph: "\u{131CB}", sign: "M17 reed" },
  f: { glyph: "\u{13191}", sign: "I9 horned viper" },
  g: { glyph: "\u{133BC}", sign: "W11 jar stand" },
  h: { glyph: "\u{13254}", sign: "O4 reed shelter" },
  i: { glyph: "\u{131CB}", sign: "M17 reed" },
  j: { glyph: "\u{13193}", sign: "I10 cobra" },
  k: { glyph: "\u{133A1}", sign: "V31 basket" },
  l: { glyph: "\u{130ED}", sign: "E23 lion" },
  m: { glyph: "\u{13153}", sign: "G17 owl" },
  n: { glyph: "\u{13216}", sign: "N35 water" },
  o: { glyph: "\u{13171}", sign: "G43 quail chick" },
  p: { glyph: "\u{132AA}", sign: "Q3 stool" },
  q: { glyph: "\u{1320E}", sign: "N29 hill" },
  r: { glyph: "\u{1308B}", sign: "D21 mouth" },
  s: { glyph: "\u{132F4}", sign: "S29 folded cloth" },
  t: { glyph: "\u{133CF}", sign: "X1 bread" },
  u: { glyph: "\u{13171}", sign: "G43 quail chick" },
  v: { glyph: "\u{13191}", sign: "I9 horned viper" },
  w: { glyph: "\u{13171}", sign: "G43 quail chick" },
  x: { glyph: "\u{133A1}\u{132F4}", sign: "V31 + S29" },
  y: { glyph: "\u{131CC}", sign: "M17A two reeds" },
  z: { glyph: "\u{13283}", sign: "O34 door bolt" },
};

export function CartoucheName({ initial = "Mira" }: { initial?: string }) {
  const [name, setName] = useState(initial);
  const id = useId();
  const letters = name
    .toLowerCase()
    .normalize("NFD")
    .replace(/[^a-z]/g, "")
    .slice(0, 12)
    .split("");

  return (
    <div className="cartouche-toy">
      <label htmlFor={id} className="clocks-label">
        Write a name
      </label>
      <input
        id={id}
        className="cartouche-input"
        value={name}
        maxLength={14}
        autoComplete="off"
        spellCheck={false}
        onChange={(event) => setName(event.target.value)}
      />
      <div className="cartouche-ring" role="img" aria-label={`${name || "An empty name"} in a playful hieroglyphic alphabet`}>
        {letters.length === 0 ? (
          <span className="cartouche-empty">…</span>
        ) : (
          letters.map((letter, i) => (
            <span key={`${i}-${letter}`} className="cartouche-sign" style={{ ["--i" as string]: i }} title={signs[letter]?.sign}>
              <span className="cartouche-glyph">{signs[letter]?.glyph}</span>
              <span className="cartouche-letter">{letter}</span>
            </span>
          ))
        )}
      </div>
      <p className="cartouche-note">
        A playful alphabet using uniliteral signs. Real Middle Egyptian spelled names differently and mostly left out vowels.
      </p>
    </div>
  );
}
