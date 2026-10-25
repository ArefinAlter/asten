import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { IbisStandard, Lotus, PapyrusPlant, Wedjat } from "../art/Figures";
import { Frieze, type FriezeKind } from "../art/Friezes";
import { Glyph, type GlyphKey } from "../art/Glyphs";
import { LunarEmblem } from "../art/Night";
import { PaletteMark } from "../art/Palette";
import { PyramidSection } from "../art/Pyramid";
import { Scales } from "../art/Scales";
import { Thoth } from "../art/Thoth";
import { CartoucheWordmark } from "../components/Brand";
import { Arrow } from "../components/Icons";
import { Answer, PageHero, SectionHead } from "../components/Page";
import { DeskMock } from "../demos/DeskMock";

const day = [
  { name: "Papyrus", token: "--papyrus", hex: "#F5EEDF", role: "Page background" },
  { name: "Papyrus raised", token: "--papyrus-raised", hex: "#FBF7EE", role: "Cards and the writing page" },
  { name: "Papyrus sunk", token: "--papyrus-sunk", hex: "#ECE2CC", role: "Alternating registers" },
  { name: "Carbon ink", token: "--ink", hex: "#1E1A15", role: "Text: the writer’s voice" },
  { name: "Rubric red", token: "--rubric", hex: "#B0351F", role: "Asten’s voice: marks and the primary action" },
  { name: "Lapis", token: "--lapis", hex: "#1F3F77", role: "Links, focus, interactive state" },
  { name: "Malachite", token: "--malachite", hex: "#2E6B4E", role: "Settled canon and success" },
  { name: "Gold", token: "--gold", hex: "#B98419", role: "Stars and ornament" },
];

const pigments = [
  { name: "Egyptian blue", hex: "#2B62B0" },
  { name: "Turquoise", hex: "#3D9A94" },
  { name: "Yellow ochre", hex: "#C98A2E" },
  { name: "Gold", hex: "#D9A631" },
  { name: "Flesh", hex: "#A9532E" },
  { name: "Natron", hex: "#FFFCF5" },
  { name: "Sand", hex: "#DCC597" },
  { name: "Night", hex: "#0F1A2E" },
];

const glyphs: { key: GlyphKey; label: string }[] = [
  { key: "scroll", label: "Desk" },
  { key: "feather", label: "Canon" },
  { key: "seated", label: "Cast" },
  { key: "moon", label: "Time" },
  { key: "hills", label: "World" },
  { key: "kit", label: "Compiler" },
  { key: "papyrus", label: "Packs" },
  { key: "door", label: "Guests" },
  { key: "house", label: "Local" },
  { key: "eye", label: "Privacy" },
  { key: "scales", label: "Diagnostics" },
  { key: "mouth", label: "Dialogue" },
  { key: "heart", label: "Fandom" },
  { key: "ibis", label: "Thoth" },
  { key: "lotus", label: "Ornament" },
  { key: "star", label: "Night" },
];

const friezes: { kind: FriezeKind; name: string }[] = [
  { kind: "blocks", name: "Block border" },
  { kind: "khekher", name: "Khekher" },
  { kind: "lotus", name: "Lotus and bud" },
  { kind: "stars", name: "Star band" },
  { kind: "water", name: "Water" },
];

const downloads = [
  { href: "/brand/asten-mark.svg", label: "Palette mark", note: "SVG, day" },
  { href: "/brand/asten-mark-night.svg", label: "Palette mark", note: "SVG, night" },
  { href: "/brand/asten-logo.svg", label: "Horizontal logo", note: "SVG, day" },
  { href: "/brand/asten-logo-night.svg", label: "Horizontal logo", note: "SVG, night" },
  { href: "/brand/asten-cartouche.svg", label: "Cartouche lockup", note: "SVG" },
  { href: "/icon-512.png", label: "App icon", note: "PNG, 512 px" },
  { href: "/og/home.png", label: "Social card", note: "PNG, 1200 × 630" },
];

export function Brand() {
  return (
    <>
      <PageHero
        title="Brand guidelines and press kit"
        lede="The Asten brand in one page: the idea, the mark, the scribe’s pigments, the type, the registers, the art, the motion, and the voice. Press can download the logo files at the bottom."
        art={<CartoucheWordmark className="brand-hero-cartouche" />}
      >
        <a href="#downloads" className="link-arrow">
          Jump to downloads <Arrow />
        </a>
      </PageHero>

      <section className="register register--sunk" aria-label="The idea">
        <div className="wrap">
          <Answer
            label="The idea"
            text="Asten is another name for Thoth, the Egyptian god of writing, who recorded the verdict at the weighing of the heart without deciding it. Egyptian scribes wrote in black and marked in red. So: the writer’s words are black ink, Asten only ever writes in red, and the author always decides. Every brand choice follows from that."
          />
        </div>
      </section>

      <section className="register" aria-labelledby="b-logo">
        <div className="wrap">
          <SectionHead
            id="b-logo"
            title="Logo: the Palette mark"
            text="A scribe’s palette seen from above, after the hieroglyph for scribe and writing (Y3): one black well for the writer, one red well for the editor, and a reed pen at rest in its slot."
          />
          <div className="logo-sheet">
            <figure className="logo-tile">
              <PaletteMark className="logo-xl" />
              <figcaption>Mark</figcaption>
            </figure>
            <figure className="logo-tile">
              <span className="brand brand-lg">
                <PaletteMark />
                <span className="wordmark">Asten</span>
              </span>
              <figcaption>Horizontal lockup (default)</figcaption>
            </figure>
            <figure className="logo-tile logo-tile--night">
              <span className="brand brand-lg">
                <PaletteMark />
                <span className="wordmark">Asten</span>
              </span>
              <figcaption>On lapis</figcaption>
            </figure>
            <figure className="logo-tile">
              <CartoucheWordmark />
              <figcaption>Cartouche (ceremonial)</figcaption>
            </figure>
          </div>
          <div className="dodont">
            <div>
              <h3>Do</h3>
              <ul>
                <li>Keep the right-hand well red in full-colour use.</li>
                <li>Give the mark one well-diameter of clear space.</li>
                <li>Use the cartouche only for ceremonial places: footer, social cards, press, splash.</li>
              </ul>
            </div>
            <div>
              <h3>Don’t</h3>
              <ul>
                <li>Swap the wells, or make both red.</li>
                <li>Add gradients, gold foil, bevels, or shadows.</li>
                <li>Set “Asten” in Papyrus, Cinzel, or any faux-Egyptian face.</li>
                <li>Put the mark inside a pyramid, an ankh, or an eye.</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="b-color">
        <div className="wrap">
          <SectionHead
            id="b-color"
            title="Colour"
            text="Egyptian painters used a small mineral palette: carbon, red and yellow ochre, Egyptian blue, malachite, gypsum white, and gold. So do we, always flat. Red is reserved: it means Asten is speaking."
          />
          <div className="swatches">
            {day.map((swatch) => (
              <figure key={swatch.token} className="swatch" style={{ "--sw": `var(${swatch.token})` } as CSSProperties}>
                <span className="swatch-chip" />
                <figcaption>
                  <strong>{swatch.name}</strong>
                  <code>{swatch.hex}</code>
                  <span>{swatch.role}</span>
                </figcaption>
              </figure>
            ))}
          </div>
          <h3 className="sub-h">Illustration pigments</h3>
          <div className="pigments">
            {pigments.map((pigment) => (
              <span key={pigment.name} className="pigment" style={{ "--sw": pigment.hex } as CSSProperties}>
                <i />
                {pigment.name} <code>{pigment.hex}</code>
              </span>
            ))}
          </div>
          <p className="fine" style={{ marginTop: 20 }}>
            Rough ratio: 70% papyrus, 15% carbon, 7% blue, 5% red, 3% gold and green. Every text pair meets WCAG 2.2 AA in both
            the day (papyrus) and night (lapis) themes. Red never carries meaning alone: every mark also has a code and words.
          </p>
        </div>
      </section>

      <section className="register" aria-labelledby="b-type">
        <div className="wrap">
          <SectionHead
            id="b-type"
            title="Typography"
            text="All self-hosted, all under the SIL Open Font License."
          />
          <div className="type-specimens">
            <div className="specimen specimen--display">
              <span className="specimen-label">The god · Fraunces</span>
              <p>Let AI be your editor.</p>
            </div>
            <div className="specimen specimen--text">
              <span className="specimen-label">The page · Literata</span>
              <p>The tide came in grey over the harbor at Veld. Mira pushed her auburn hair out of her eyes and listened for the bell.</p>
            </div>
            <div className="specimen specimen--mono">
              <span className="specimen-label">The scribe’s notation · IBM Plex Mono</span>
              <p>
                <span className="chip chip--code">CANON-E013</span> Mira · hair_color · auburn
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="b-layout">
        <div className="wrap">
          <SectionHead
            id="b-layout"
            title="Layout and ornament"
            text="Egyptian walls are organised in horizontal registers, each standing on a ground line. Our pages are too. Ornament frames content and never sits behind running text."
          />
          <div className="frieze-sheet">
            {friezes.map((frieze) => (
              <figure key={frieze.kind}>
                <Frieze kind={frieze.kind} />
                <figcaption>{frieze.name}</figcaption>
              </figure>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="b-art">
        <div className="wrap">
          <SectionHead
            id="b-art"
            title="Illustration"
            text="Original SVG drawings in the Egyptian composite view: head and legs in profile, shoulders and eye frontal, flat pigment fills, carbon outlines, ground lines, no gradients and no perspective."
          />
          <div className="art-sheet">
            <figure>
              <Thoth />
              <figcaption>Thoth the scribe</figcaption>
            </figure>
            <figure>
              <Scales tilt={-8} />
              <figcaption>The scales of Ma’at</figcaption>
            </figure>
            <figure>
              <IbisStandard />
              <figcaption>Ibis on a standard</figcaption>
            </figure>
            <figure>
              <Wedjat />
              <figcaption>Wedjat (protection)</figcaption>
            </figure>
            <figure>
              <PyramidSection />
              <figcaption>The world below the sand</figcaption>
            </figure>
            <figure className="art-night">
              <LunarEmblem />
              <figcaption>Lunar emblem</figcaption>
            </figure>
            <figure>
              <Lotus />
              <figcaption>Blue lotus</figcaption>
            </figure>
            <figure>
              <PapyrusPlant />
              <figcaption>Papyrus</figcaption>
            </figure>
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="b-icons">
        <div className="wrap">
          <SectionHead
            id="b-icons"
            title="Icons"
            text="Each icon is drawn from a hieroglyph that already meant the thing: the papyrus roll for writing, the feather for truth, the house plan for home. 32-unit grid, round joins, one optional flat fill."
          />
          <div className="glyph-sheet">
            {glyphs.map((glyph) => (
              <figure key={glyph.key}>
                <Glyph name={glyph.key} />
                <figcaption>{glyph.label}</figcaption>
              </figure>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="b-motion">
        <div className="wrap split">
          <div>
            <SectionHead
              id="b-motion"
              title="Motion"
              text="Motion should feel like ink going down: deliberate, then still. Black first, red after a beat, because Asten never interrupts."
            />
          </div>
          <ul className="motion-list">
            <li>
              <strong>Draw</strong>
              <span>Strokes ink on in 600–1200 ms with an ink easing.</span>
            </li>
            <li>
              <strong>Rise</strong>
              <span>Registers rise 16 px and fade in as you reach them.</span>
            </li>
            <li>
              <strong>Weigh</strong>
              <span>The scales tip with a damped settle when a page disagrees.</span>
            </li>
            <li>
              <strong>Ambient</strong>
              <span>Stars twinkle, the moon glows, the pen writes: slow loops, never faster than two seconds.</span>
            </li>
            <li>
              <strong>Still</strong>
              <span>Reduced-motion settings remove all of it. Every animated state has a still twin.</span>
            </li>
          </ul>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="b-voice">
        <div className="wrap">
          <SectionHead id="b-voice" title="Voice" text="Calm, precise, and plain. Asten speaks like a careful reader, not a salesperson." />
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th scope="col">We say</th>
                  <th scope="col">We don’t say</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>“Mira’s hair was settled as auburn.”</td>
                  <td>“Error: inconsistency detected in character profile.”</td>
                </tr>
                <tr>
                  <td>“Keep my sentence.”</td>
                  <td>“Ignore warning.”</td>
                </tr>
                <tr>
                  <td>“No model runs in Asten today.”</td>
                  <td>“AI-powered continuity magic.”</td>
                </tr>
                <tr>
                  <td>“It’s a pre-alpha, and the installers are unsigned.”</td>
                  <td>“Coming soon to revolutionize your writing!”</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="fine" style={{ marginTop: 18 }}>
            Words we use: notice, mark, weigh, record, settle, canon, your call. Words we avoid: generate, supercharge, unleash,
            magic, seamless, effortless, game-changing. Headings describe what follows, in plain words: no kicker labels or
            numbering above them, no slogan or punchline headings, and no copy that tells the reader to hover, click, or try
            something.
          </p>
        </div>
      </section>

      <section className="register" aria-labelledby="b-product">
        <div className="wrap">
          <SectionHead
            id="b-product"
            title="In the product"
            text="Inside the desk, ornament steps back. The page is plain papyrus and carbon ink. Asten’s voice is red, in the margin. Settled canon is green. Names sit in cartouches. Art lives at the edges: empty states, onboarding, the splash."
          />
          <DeskMock />
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="b-respect">
        <div className="wrap wrap-narrow prose">
          <h2 id="b-respect">Using Egyptian art with care</h2>
          <p>
            No mummy jokes, curses, or pharaoh kitsch. No invented hieroglyphic “text”: when we show real signs, we name them, and
            the cartouche name toy is labeled as a playful alphabet. Thoth is drawn with dignity, with his ibis head, lunar disk,
            palette, and pen. Our art is original, made for Asten, in the spirit of the tradition. See <Link to="/name">the name</Link>.
          </p>
        </div>
      </section>

      <section className="register" id="downloads" aria-labelledby="b-dl">
        <div className="wrap">
          <SectionHead
            id="b-dl"
            title="Press kit downloads"
            text="Use these files unaltered when writing about Asten. For anything else, see the trademark policy or write to us."
          />
          <div className="related">
            {downloads.map((item) => (
              <a key={item.href} href={item.href} download>
                <span>
                  {item.label} <small className="fine">{item.note}</small>
                </span>
                <Arrow />
              </a>
            ))}
          </div>
          <p className="fine" style={{ marginTop: 18 }}>
            Boilerplate: “Asten is a local-first writing app from Verne that remembers the facts of a story and marks contradictions
            in red, without ever writing prose. Its rule: let AI be your editor, not your writer.” See also the{" "}
            <Link to="/trademark">trademark policy</Link>.
          </p>
        </div>
      </section>
    </>
  );
}
