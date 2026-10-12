import { Link } from "react-router-dom";
import { Scales } from "../art/Scales";
import { Diagnostic, FaqList, Sections } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Inline } from "../components/Inline";
import { Answer, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import { howSteps } from "../content/home";
import type { QA, Section } from "../content/types";
import { DeskMock } from "../demos/DeskMock";
import { WeighDemo } from "../demos/WeighDemo";

const details: Section[] = [
  {
    id: "step-1",
    heading: "1. Write, with nothing in the way",
    blocks: [
      { kind: "p", text: "Create a project (novel, screenplay, non-fiction, or something else) and start a scene. Asten doesn’t ask for a synopsis, a character sheet, or a map. The [binder](/features/desk) holds chapters, scenes, notes, and sources; the page is plain and quiet." },
      { kind: "p", text: "If you already have a draft, import it from Markdown, plain text, Fountain, HTML, or .docx." },
    ],
  },
  {
    id: "step-2",
    heading: "2. Settle the facts that matter",
    blocks: [
      { kind: "p", text: "A detail becomes canon when you settle it. Each [fact](/glossary#fact) records who it’s about, what it says, how sure you are, when in story time it’s true, and where it came from." },
      { kind: "terms", items: [
        { term: "Mira · hair · auburn", text: "Confirmed, established, settled in chapter 2." },
        { term: "Oren · status · dead", text: "Confirmed, from chapter 9 onward." },
        { term: "Veld · east of · the reef", text: "Settled from the atlas." },
        { term: "The guild · founded · year 300", text: "Contested: three characters disagree, and that’s the point." },
      ] },
      { kind: "p", text: "You can settle facts from the cast, from a world entry you mark *settled*, from a wiki page’s attributes, or with an inline note like `[[mira.hair_color=auburn]]`. Today Asten never pulls facts out of your prose on its own." },
    ],
  },
  {
    id: "step-3",
    heading: "3. Asten notices, in red",
    blocks: [
      { kind: "p", text: "Asten’s [compiler](/features/compiler) weighs your project against its canon. When something doesn’t balance, a red mark appears in the margin beside the sentence, with a stable code and both sides of the disagreement." },
    ],
  },
  {
    id: "step-4",
    heading: "4. You decide",
    blocks: [
      { kind: "p", text: "A mark is a question, not a correction. Every mark offers choices, and none of them touch your sentence unless you do:" },
      { kind: "list", items: [
        "**Keep my sentence.** Silence the mark. The suppression and your reason stay with the project.",
        "**Update canon.** The page was right; the record changes.",
        "**Record a change.** Give facts a span in story time: auburn until chapter 12, black after.",
        "**Mark it contested.** For lies, legends, and unreliable narrators. Contested facts never raise errors.",
        "**Deprecate the old fact.** It stays in history but stops being compared.",
      ] },
    ],
  },
];

const faqs: QA[] = [
  { q: "How does Asten find contradictions?", a: "Asten compares the facts you have settled with each other, with your timeline, and, for hair and eye colors, with the words near a character’s name in your prose. It uses deterministic rules with stable codes, not an AI model." },
  { q: "Does Asten change my text when it finds a problem?", a: "No. Asten only adds a red mark in the margin. Your text changes only if you edit it yourself." },
  { q: "What if I don’t want to settle any facts?", a: "Then Asten is simply a calm manuscript desk with a binder, comments, page view, and compile. The continuity checks only speak once there is canon to compare against." },
];

export function HowItWorks() {
  return (
    <>
      <PageHero
        eyebrow="How it works"
        title="Write. Settle. Notice. You decide."
        lede="Asten builds a record of your story as you go, weighs each page against it, and marks what doesn’t balance. The verdict, like the pen, stays with you."
        art={<Scales tilt={-7} />}
      >
        <div className="btn-row">
          <Link to="/download" className="btn btn--primary" data-magnetic>
            Get early access
          </Link>
          <a href="#try" className="link-arrow">
            Try the demo <Arrow />
          </a>
        </div>
      </PageHero>

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten works in four steps. You write freely; you settle the details that matter as facts; Asten’s deterministic compiler marks any page that disagrees with those facts, in red, with evidence; and you decide whether to keep the sentence, update canon, record a change, or silence the rule. Asten never edits your text." />
          </div>
          <ol className="steps" style={{ marginTop: 40 }} data-reveal>
            {howSteps.map((step, i) => (
              <li key={step.title} className="step" id={`step-${i + 1}`}>
                <span className="step-num" aria-hidden="true">
                  {Array.from({ length: i + 1 }, (_, k) => (
                    <i key={k} />
                  ))}
                </span>
                <h2 className="h3">{step.title}</h2>
                <p>
                  <Inline text={step.text} />
                </p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="register" aria-label="The four steps in detail">
        <div className="wrap split-prose">
          <div className="prose" data-reveal>
            <Sections sections={details.slice(0, 2)} />
          </div>
          <div data-reveal>
            <DeskMock />
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="codes-h">
        <div className="wrap">
          <SectionHead id="codes-h" eyebrow="What a mark looks like" title="Three rules ship today." text="Each mark has a stable code that means the same thing every time." />
          <div className="card-grid card-grid--3">
            <div data-reveal>
              <Diagnostic code="CANON-E013" title="Mira’s hair was settled as auburn." lines={["Chapter 2: Mira · hair_color · auburn", "Chapter 19: “…her black hair…”", "Keep · Update canon · Add change"]} />
            </div>
            <div data-reveal>
              <Diagnostic code="CANON-E101" title="Oren acts after a settled death." lines={["Chapter 9: Oren · status · dead", "Chapter 14: “Oren pours the tea”", "Mark flashback · Move the death · Silence"]} />
            </div>
            <div data-reveal>
              <Diagnostic code="CANON-E201" title="Story time runs backward inside a continuous run." lines={["Ch. 11 “The harbor burns”: day 17", "Ch. 12 “The morning after”: day 4", "Reorder · Retcon a date · Mark it a flashback"]} />
            </div>
          </div>
        </div>
      </section>

      <section className="register" aria-label="Deciding">
        <div className="wrap">
          <div className="prose" data-reveal>
            <Sections sections={details.slice(2)} />
          </div>
        </div>
      </section>

      <section className="register register--raised" id="try" aria-labelledby="try-h">
        <div className="wrap">
          <SectionHead id="try-h" eyebrow="Try it" title="Weigh a sentence yourself." />
          <div className="demo-frame" data-reveal data-llms="skip">
            <WeighDemo />
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="hfaq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="hfaq-h" eyebrow="Questions" title="About the process" />
          <FaqList items={faqs} />
        </div>
      </section>

      <Related paths={["/features/canon", "/features/time", "/features/compiler", "/journal/continuity-in-a-long-novel"]} />
      <CtaBand />
    </>
  );
}
