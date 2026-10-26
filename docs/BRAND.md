# Asten brand guidelines

Version 1.1 · 26 October 2026 · Owner: Verne Ltd.

This file is the source of truth for how Asten looks, sounds, and moves. It covers
the public website (`apps/www`), the product (`apps/web`, `apps/desktop`), and
anything else that carries the Asten name. The tokens live in
`packages/brand/src/tokens.css`. The public version of this page is `/brand` on
the website.

---

## 1. The idea

**Asten is another name for Thoth**, the Egyptian god of writing. Thoth invented
writing, kept the records of the gods, reckoned time by the moon, and stood beside
the scales at the Weighing of the Heart to record the result. He recorded the
verdict. He did not decide it.

That is the product:

- Asten **keeps the record** of your story: the facts you settle, the clocks, the
  cast, the world.
- Asten **weighs** each new page against that record and marks what doesn't
  balance.
- Asten **does not decide**, and **does not write**. The author holds the pen and
  the verdict.

### The palette metaphor

An Egyptian scribe carried a palette with two cakes of ink: **black** for the text
and **red** to mark it (headings, verse points, dates, corrections). Asten keeps
the same rule:

> **Your words are black ink. Asten only ever writes in red.**

Everything the author writes is theirs and stays untouched. Everything Asten adds
(a mark in the margin, a code, a note, a suggestion) is red, and it is always
optional. That rule appears in the logo, the product UI, the website, and the
copy.

### Brand promise

> **Let AI be your editor, not your writer.**

Supporting lines (use one at a time, never stacked):

- Asten keeps the record. You keep the pen.
- Your words in black. Our marks in red.
- A writing desk with a memory.
- It weighs. You decide.

### Positioning statement

For creative writers who are curious about AI but protective of their voice,
Asten is a local-first writing environment that remembers the facts of a story
and marks the moment a draft contradicts itself, without writing a word.
Unlike AI story generators, it never touches your prose. Unlike word processors,
it knows your canon. Unlike cloud tools, your manuscript never leaves your
computer.

### Pillars

1. **The pen stays yours.** Asten never generates prose. Anything AI touches in
   the future is a proposal the author confirms.
2. **Every mark shows its evidence.** A diagnostic names the rule, the
   sentence, and the settled fact it disagrees with. (Ma'at: truth and balance.)
3. **The house is yours.** Local-first. No account, no sync, no telemetry, and no
   training on your words.
4. **Quiet craft.** No doctrine and no nagging. Method packs are optional,
   craft hints are off by default, and any rule can be silenced with a reason.
5. **Honest about the work.** Say what ships today and what is planned. Never
   pretend a roadmap item exists.

---

## 2. Audience

Primary: **creative writers who tinker with AI.** Novelists, fan-fiction
writers, worldbuilders, screenwriters, and game and campaign writers. They have
tried ChatGPT, Claude, or an AI writing app, and some of them felt their voice
flatten. They want help with the tedious part, which is remembering what they already
decided, and they want none with the part that makes the work theirs.

Secondary: **AI tinkerers and builders.** They care that the tool runs locally,
that the rules are deterministic and documented with stable codes, that the
architecture is plain (Rust, SQLite, a loopback API), and that any future model
can be local or bring-your-own. (The source is public on GitHub under the MIT
License; say so plainly, and say that the names and logos are not covered by it.)

What they fear: losing their voice, their work being used to train models,
cloud lock-in, nagging "grammar police" tools, and a 40-field character form
before they can type a sentence.

What they want: to finish the book, to have readers not catch the continuity
slip, and to keep ownership.

---

## 3. Voice

Asten speaks like **a good scribe**: calm, precise, warm, slightly wry. It is a
peer who reads carefully, not a guru and not a hype machine.

| We are | We are not |
| --- | --- |
| Plain. Short sentences, real words. | Jargon-first ("leverage our NLP pipeline"). |
| Concrete: Mira's auburn hair, the mentor who died in Act One. | Abstract ("unlock your creative potential"). |
| Honest about limits ("It's a pre-alpha, and the installers are unsigned."). | Over-promising ("AI that understands your story"). |
| Respectful of the writer's authority. | Prescriptive ("Fix this weak sentence"). |
| Lightly mythic, once per page at most. | Costume-y ("Behold, mortal!"). |

### Words we use

notice, mark, point at, weigh, record, settle, canon, draft, scene, page, margin,
evidence, your call, suppress, local, yours.

### Words we avoid

generate, write for you, co-write, supercharge, unleash, magic, revolutionary,
seamless, effortless, 10x, game-changing, "AI-powered" (as a headline), "smart".

### Rules for AI talk

- Asten **today** uses deterministic rules over facts the author confirms. No
  model runs, local or remote. Never imply otherwise.
- Planned AI work is described as **planned**, as **proposals the author
  confirms**, and as **local or bring-your-own-key**. Link to `/roadmap`.
- Never claim Asten "understands" a story. It compares settled facts.
- No manuscript is ever used to train a model. Say it plainly.

### Microcopy patterns

- Diagnostic title: what disagrees, in one line. "Mira's hair was settled as
  auburn."
- Diagnostic body: where the two sides are. "Chapter 2 says auburn. This page
  says black."
- Actions are verbs the author owns, worded as the engine words them: *Keep my
  sentence*, *Add an event*, *Retire "auburn"*, *Mark the scene as a memory*,
  *Silence this mark*.
- Never use "error" in front of the author for craft hints. Hints are "notes".
- Buttons: sentence case, two or three words. "Get Asten", "See how it works".

---

## 4. Logo

### The Palette mark

The mark is a scribe's palette seen from above: a tall rounded slate with
**two ink wells**, one black and one red, and a **reed pen** resting in its slot.
It is drawn from the hieroglyph for *scribe* and *writing* (𓏞, Gardiner Y3),
a scribe's kit of palette, water pot, and pen. The two wells carry the brand rule:
black for the writer, red for the editor.

- Construction: 24 × 32 unit grid. Slate corner radius 6. Wells are 6-unit
  circles centred at y = 8, offset 3.5 left and right of centre. The pen slot
  runs from y = 14 to y = 28 and holds a 2-unit reed with a cut nib.
- Minimum size: 16 px tall (favicon variant drops the slot shading).
- Clear space: one well-diameter on every side.

### Wordmark

"Asten" set in Fraunces, weight 600, optical size 144, tracking −1%. The A's
apex is left as drawn. Do not redraw letters.

### Lockups

1. **Horizontal**: mark + wordmark, gap equal to one well-diameter. Default.
2. **Cartouche**: the wordmark inside a cartouche (a rounded oval with a tie bar
   at the right end). Use it for ceremonial places only: the footer, OG images,
   the press kit, and the product splash screen.
3. **Mark only**: app icon, favicon, social avatar.

### The app icon

The application icon is the mark only, on the day page colour (`#F5EEDF`) as a
square with no rounded plate (this guide defines none): the favicon geometry of
the Palette (heavier strokes, so it holds at 16 px), the slate at 71% of the
plate's height (clear space of one well-diameter on every side), larger at 32 px
and below (88%, 80% up to 64 px), and 60% in the maskable web icon so the whole
slate sits inside the circle every platform mask keeps. It is drawn by
`apps/www/scripts/app-icons.mjs` from the mark's own geometry; do not edit the
PNGs, the `.ico` or the `.icns` by hand. The master is
`apps/desktop/src-tauri/icon-source/asten-icon.svg`.

### Colourways

- Carbon on papyrus (default).
- Papyrus on lapis (night).
- The right-hand well is **always red** in full-colour versions. In one-colour
  versions both wells are outlined and the right well is filled.

### Don't

- Don't swap the well colours or make both wells red.
- Don't add gradients, bevels, gold foil effects, or drop shadows.
- Don't put the mark inside a pyramid, an ankh, or an eye.
- Don't set the wordmark in Papyrus, Cinzel, or any "Egyptian-looking" display face.
- Don't rotate, stretch, or outline the wordmark.

---

## 5. Colour: the scribe's pigments

Egyptian painters worked with a small mineral palette: carbon black, red and
yellow ochre, Egyptian blue (the first synthetic pigment), malachite green,
gypsum white, and gold. Asten uses the same set, as flat colour.

### Day (papyrus), default

| Token | Hex | Role |
| --- | --- | --- |
| `--papyrus` | `#F5EEDF` | Page background |
| `--papyrus-raised` | `#FBF7EE` | Cards, raised surfaces, the writing page |
| `--papyrus-sunk` | `#ECE2CC` | Alternating registers, wells, inputs |
| `--rule` | `#D9CAA9` | Hairlines and borders |
| `--rule-strong` | `#BFAB82` | Ground lines and emphasised borders |
| `--ink` | `#1E1A15` | Text (carbon black) |
| `--ink-2` | `#4A4136` | Secondary text |
| `--ink-3` | `#6E6253` | Muted text (AA on papyrus) |
| `--rubric` | `#B0351F` | **Asten's voice**: marks, diagnostics, primary CTA |
| `--rubric-deep` | `#8C2814` | Hover and pressed rubric |
| `--lapis` | `#1F3F77` | Links, focus, interactive state |
| `--blue` | `#2B62B0` | Egyptian blue, used in illustration and ornament |
| `--malachite` | `#2E6B4E` | Settled canon, confirmed, success |
| `--turquoise` | `#3D9A94` | Faience accents in art |
| `--ochre` | `#C98A2E` | Yellow ochre in art |
| `--gold` | `#B98419` | Stars, ornament highlights |
| `--flesh` | `#A9532E` | Figure skin in illustration |
| `--natron` | `#FFFCF5` | White fills in illustration (linen, kilts) |

### Night (lapis)

Night is the moon's palette: Thoth is also a moon god. It is used for the dark
theme and for "ceiling" sections on the website (deep blue with gold stars, as
in New Kingdom tomb ceilings). On the website the dark theme is **lamplight**:
the page is lapis and ink black, painted plates are dimmed a little and each
sits in a pool of warm light, and the lamp flickers by a small amount of noise,
never on a timer.

| Token | Hex | Role |
| --- | --- | --- |
| `--papyrus` | `#0F1A2E` | Page background |
| `--papyrus-raised` | `#15233D` | Cards |
| `--papyrus-sunk` | `#0B1424` | Registers |
| `--rule` | `#2A3A5A` | Hairlines |
| `--ink` | `#F1E7D2` | Text |
| `--ink-2` | `#CBBFA6` | Secondary |
| `--ink-3` | `#9E937F` | Muted |
| `--rubric` | `#E8694C` | Asten's voice |
| `--lapis` | `#9BBDF2` | Links and focus |
| `--malachite` | `#6CC59A` | Settled |
| `--gold` | `#E3B341` | Stars |

### Ratios and rules

- Roughly 70% papyrus, 15% carbon, 7% blue, 5% red, 3% gold and green.
- **Red is reserved.** It means "Asten is speaking": a mark, a diagnostic, the
  primary action. Don't use red for decoration in UI, and never for the author's own text.
- **Green means settled.** Use malachite only for confirmed canon and success.
- Colour is never the only carrier of meaning. Every red mark has a code and
  words; every green state has a label.
- All text pairs meet WCAG 2.2 AA: body text 4.5:1 or better, large text 3:1 or better.

---

## 6. Typography

| Voice | Face | Use |
| --- | --- | --- |
| **The god** (display) | Fraunces Variable, optical size axis | Headlines, the wordmark, numerals in art |
| **The page** (text) | Literata Variable | Body copy, the manuscript page in the product |
| **The scribe's notation** | IBM Plex Mono | Diagnostic codes (`CANON-E013`), labels, keyboard and paths |

- All fonts are self-hosted (OFL). The website loads nothing from a font CDN.
- Headline scale (website): 76 / 56 / 40 / 30 / 24 px on desktop, fluid with
  `clamp()`. Body 18 px / 1.65. Small 15 px. Label 12 px mono, +0.12em tracking,
  uppercase.
- Headline italics in Fraunces carry emphasis, usually the word "editor" or
  "yours".
- Measure: 62–70 characters for reading text.
- Diagnostic codes are always mono and always red, in a small cartouche.

---

## 7. Layout: registers

Egyptian walls are organised in **registers**: horizontal bands, each standing
on a ground line. The website and the product use the same idea.

- Each section is a register. It sits on a **ground line** (a 2 px rule above a
  1 px rule, 3 px apart) or on a frieze.
- Alternate registers between `--papyrus` and `--papyrus-sunk`. Use night
  (lapis) registers at most twice per page.
- Grid: 12 columns, 24 px gutters, max content width 1200 px, reading column
  680 px.
- Spacing scale (px): 4, 8, 12, 16, 24, 32, 48, 64, 96, 128.
- **Hierarchy of scale**: like Egyptian art, the most important figure is the
  largest. One hero element per register.

---

## 8. Ornament

Ornament frames content. It never sits behind running text.

- **Friezes**: lotus bud-and-flower, *khekher* (the knotted reed frieze at the
  top of tomb walls), star band, and water zigzag. Use them as register
  borders, 16–28 px tall.
- **Cartouche**: an oval with a tie bar at one end. It encloses a name: the
  wordmark, a character in the cast, a diagnostic code, or a primary button.
- **Columns**: vertical bands with glyph columns as side borders on wide screens.
- **Stars**: five-pointed, gold on lapis, for night registers only.

### Respect the source

Egyptian art belongs to Egypt's heritage and is still studied and loved. Rules:

- No mummy jokes, curses, "pharaoh" kitsch, or pyramid-scheme puns.
- Don't invent fake hieroglyphic "text". When we use real signs (the cartouche
  name toy, the logo's source sign), we say what they are and that the name
  toy is a playful alphabet, not Middle Egyptian.
- Depict Thoth with dignity: the ibis head, lunar disk, palette, and reed pen.
- Credit the tradition on `/name`.

---

## 9. Illustration

Two kinds of picture sit together on the site. They are never mixed in one frame.

### Painted plates

Public-domain and CC0 works from museum and library collections, cropped, cut,
graded to one light and set on the paper. The scribe's desk is where they live.

- **Only works that need no permission.** Public domain, or released by the
  holder under CC0 (the Met's Open Access programme, the National Gallery of
  Art, the Cleveland Museum of Art, the Yale Center for British Art, the British
  Library). Nothing is generated, redrawn or painted over beyond the listed
  treatments. Never AI-made art.
- **One light.** The build gains the highlights toward warm paper, eases the
  saturation, lifts the blacks a little and lays a fine grain over everything,
  so a Hudson River sky, a New Kingdom papyrus, a faience water clock and a
  Dutch still life can share a page.
- **Four shapes.** A *torn sheet* for paintings and prints; an *arch* (a round
  top over a torn foot) for a window; a *band* for a strip across the page;
  a *cut-out* for an object or a portrait panel, cut from its studio backdrop
  and standing on the paper alone. Cut edges are shrunk and feathered so no
  backdrop colour rims them. Scale and crop vary from page to page.
- **Every page has its own plate.** Every page, every feature, every audience
  and every journal post has a plate, and the same plate is not used on two
  pages. A work may supply two different crops.
- **The work matches the subject.** Weighing is a balance (Hunefer, Vermeer,
  Metsys); time is a water clock, a sundial, or the same street painted at
  different hours; the cast are portraits painted for mummies; a map for the
  worldbuilders; threshing for the compiler; a cloud study for a page that is
  not there.
- **No captions, no credit lines beside a picture.** Not under the hero window,
  not on a journal card, not in a margin, not in the footer text. The credit for
  every work (title, maker, date, holder, object number, licence, link) is on
  `/credits`, which is generated from the same data as the pictures, and the
  footer carries one small plain link, "Art credits". Every picture has `alt`
  text that says what it shows. (Asten's own marginalia, the handwritten notes
  beside the manuscript text in the product and on the home page, are part of the
  design and stay: they are not credits.)
- **The paintings are present, not thumbnails.** A plate at the head of a page
  takes half the width of the head. Pages that open with a band get a strip laid
  across the whole page, edge to edge. A journal card is a painting cropped to
  fill its slot, with the object kept in view (`focus`). Do not shrink a painting
  into a vignette.
- **No text in a plate, and nothing behind running text.** Plates sit beside or
  above the words. The one at the head of a page loads eagerly and is preloaded;
  every other loads lazily and has its width and height in the markup.
- **Budgets.** The hero window is at most 300 KB and a plate at most about
  150 KB; the home page, with all its plates, stays near 1.5 MB.
- **Reproducible.** `art/build.py` makes every file from `art/works.json` (the
  works) and `art/plates.json` (the crops and the pages). It also writes
  `apps/www/src/personality/art-manifest.json`, `art/SOURCES.md` and the SHA-256
  lock `art/sources.lock.json`; the table on `/credits` is made from the same
  manifest. To add a plate, add the work, add the spec, run the
  build.
- **At night** plates are dimmed (brightness 0.82) and lit by a pool of lamplight.

### Drawn ornaments

Original SVG in the Egyptian **flat canon**:

- Composite view: head and legs in profile, shoulders and eye frontal.
- Flat pigment fills, carbon outlines (1.5–2.5 px at 1×), no gradients, no
  shading, no perspective.
- Figures stand on a ground line. Scenes sit in registers.
- Colours come only from the pigment tokens.
- Subjects: Thoth the scribe, the ibis, the scales and Ma'at's feather, the
  moon and stars, the papyrus scroll, the palette, lotus and papyrus plants,
  the pyramid in section (most of a world sits below the sand), the wedjat eye
  (protection, for privacy), and the Nile.
- In SVG, colour comes from CSS classes bound to tokens (`.f-blue`,
  `.s-ink`, and so on) so art follows day and night themes. Standalone files
  (favicon, OG, press kit) inline hex values.

### The margin hand

Asten's notes in the margin are set in **Caveat** (SIL OFL 1.1, self-hosted),
in the rubric red, a little smaller than the text beside them. It is used only
for a note Asten writes beside a page, never for headings or buttons. The red
pencil marks beside it are drawn, not set: see section 11.

---

## 10. Iconography

Icons are drawn from hieroglyphs that already mean the thing:

| Concept | Sign | Source |
| --- | --- | --- |
| Desk / manuscript | Papyrus roll, tied | Y2, "writing, abstract" |
| Cast | Seated figure | A1 |
| Time | Moon over sun disk | N11 + N5 |
| World | Hill-country | N25 |
| Canon / truth | Feather of Ma'at | H6 |
| Compiler | Scribe's kit | Y3 |
| Packs | Papyrus clump | M16 |
| Guests (Word/Docs) | Doorway | O31 |
| Local / home | House plan | O1, *pr* |
| Privacy | Wedjat eye | D10 |

Grid 32 × 32, 2 px stroke at 1×, round caps and joins, optional flat fill in a
single pigment.

---

## 11. Motion: the hand and the lamp

Motion should feel like a hand: deliberate, a little uneven, then still.

- **Draw.** The red pencil is a filled brush outline revealed by a mask whose
  centre line is drawn with `stroke-dashoffset`. The line wobbles (an SVG
  turbulence displacement), its width swells and thins, it overshoots a little,
  and no two strokes take the same time: the pace comes from seeded noise and
  eases out like a spring. Strokes are drawn again, erased and redrawn, when the
  words under them change or when the reader hovers. Elsewhere strokes draw on
  with `stroke-dashoffset` over 600–1200 ms.
- **Black, then red.** In any demo the author's text appears first and Asten's
  red mark appears after a beat (≥ 300 ms). Asten never interrupts.
- **Weigh.** The scales are a damped spring (stiffness 90, damping 6.5) with the
  pans on softer springs. A heavier claim tips the beam, it overshoots by up to
  4° and settles. The tilt is driven by what the example says.
- **Rise and turn.** Registers enter with a 14 px rise and fade, 700 ms,
  staggered 70 ms. Sheets lift over one another at the seam between sections, as
  pages turn (scroll-driven where the browser supports it).
- **Lamp.** At night the page is lit by a lamp whose flicker is smoothed noise
  on a CSS variable, small enough not to be noticed as motion.
- **Nothing loops on a timer.** Motion comes from a spring, from noise or from a
  hand. It runs only while on screen and while the tab is visible, and animation
  frames are requested only while something is moving.
- `prefers-reduced-motion: reduce` removes all movement: every drawing arrives
  complete (the pencil circle, the margin note, the beam at its resting tilt) and
  every animated state has a still equivalent. Without JavaScript the red marks
  are a plain red underline.
- Interactions answer within 100 ms.

### Signature microinteractions

1. Cartouche buttons: the tie bar slides and the outline redraws on hover.
2. Links: a red reed stroke draws under the text.
3. Nav progress: a red rubric line grows across the top as you read.
4. Theme toggle: a moon that waxes to full (night) and wanes to a crescent (day).
5. Weigh-a-sentence demo: the scales tip when the page disagrees with canon.
6. Cartouche name writer: a name becomes a cartouche of uniliteral signs.
7. Pyramid in section: hovering a chamber reveals the part of the world it holds.
8. Lotus FAQ: the lotus opens when the answer opens.

---

## 12. Product application (the desk)

The website is the foyer. The desk is the room where the work happens. The same
brand applies there, with less ornament.

- **The page is sacred.** No ornament, texture, or art inside the writing
  surface. `--papyrus-raised` page, Literata 18–20 px, carbon ink.
- **Red is Asten.** Diagnostics are red rubric marks in the margin with a mono
  code cartouche. The author's text is never recoloured, only underlined with a
  thin red reed stroke.
- **Green is settled.** Confirmed facts carry a malachite seal; proposed facts
  are outlined only.
- **Names are cartouches.** Entities in the cast and in hover cards render in a
  cartouche chip.
- **Two registers for time.** The timeline stacks reading order above story
  time, like two registers on a wall, with moon-phase ticks. Crossing lines
  show flashbacks.
- **The Problems panel is "The Scales".** Its empty state is a balanced
  scale: "Nothing disagrees with your canon."
- **Chrome**: binder and panels on `--papyrus`, separated by ground lines.
  Panel headers use mono labels.
- **Night theme** uses the lapis palette; the page becomes `#15233D` with
  `#F1E7D2` ink.
- **Art lives at the edges**: empty states, onboarding, the splash, and the
  about box.
- **Motion** in the desk is limited to the draw-in of red marks (300 ms) and
  the scales on the Problems tab. No ambient motion near the page.

---

## 13. Accessibility

- WCAG 2.2 AA contrast for every text pair in both themes.
- Visible focus: a 2 px lapis outline at 3 px offset on every interactive element.
- Every decorative SVG is `aria-hidden`. Every meaningful SVG has a title.
- Interactive demos are keyboard operable and announce results in a polite live
  region.
- Reduced motion is honoured everywhere, and the page works without JavaScript.
- Painted plates have alt text that says what they show. A plate never carries
  text of its own, and there are no captions; credits are on `/credits`.
- Diagnostics are never colour-only.

---

## 14. Legal name and attribution

- Product: **Asten**. Publisher: **Verne Ltd.** ("Verne" in running copy is fine
  after the first mention on a legal page.)
- Footer credit: "Designed and built by Verne Ltd.", linking to
  `https://vernesystems.com`.
- The registered address and company number on the legal pages come from
  `https://vernesystems.com/legal/company`. Don't invent or alter them, and
  don't add a governing law that Verne Ltd. has not supplied.
