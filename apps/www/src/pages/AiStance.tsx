import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { Thoth } from "../art/Thoth";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Answer, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import type { QA } from "../content/types";

const rules = [
  { title: "Editor, never writer", text: "No feature will generate, continue, rewrite, or autocomplete your prose. Not as an option, not as an experiment." },
  { title: "Proposals only", text: "AI may propose a fact, a link, or an explanation. Nothing it suggests becomes canon until you confirm it." },
  { title: "Evidence attached", text: "Every proposal cites the sentence it came from, so you can check it in one glance." },
  { title: "Local first", text: "Where a model is practical on your own machine, that is the default. Your manuscript stays home." },
  { title: "Your key, your choice", text: "If you choose a cloud model, you choose the provider, you hold the key, and Asten tells you plainly what will be sent." },
  { title: "One switch, all off", text: "AI features can be switched off completely. Asten remains a full desk and a full compiler without them." },
  { title: "Never training data", text: "Your manuscript is never used to train any model, by Verne or anyone Verne works with." },
];

const faqs: QA[] = [
  { q: "Does Asten use AI today?", a: "No. Asten runs no AI model today, local or remote. Its continuity checks are deterministic rules over facts you confirm." },
  { q: "Why would a writing app refuse to write?", a: "Because the writing is the part that makes the work yours. Asten aims at the part writers find tedious: remembering hundreds of settled details and noticing when a page drifts from them." },
  { q: "Can I use ChatGPT or Claude with Asten?", a: "You can use any assistant you like alongside Asten. Asten doesn’t connect to them and won’t send them your manuscript. What you paste into another tool is governed by that tool’s terms." },
  { q: "Will Asten’s future AI features cost money?", a: "Pricing hasn’t been decided. The commitments on this page don’t depend on price: proposals only, local first, your key, a full switch-off, and no training on manuscripts." },
];

export function AiStance() {
  return (
    <>
      <PageHero
        eyebrow="Our AI stance"
        title={
          <>
            The pen stays in <em>your</em> hand.
          </>
        }
        lede="AI is remarkable at remembering, comparing, and pointing. It is a poor substitute for your voice. So Asten draws one line and keeps it: AI can be your editor. It can never be your writer."
        art={<Thoth flip />}
      />

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten never generates prose and never trains on manuscripts. Today it runs no AI model at all; its checks are deterministic rules. Planned AI features will only propose facts with cited evidence for the author to confirm, will run locally where practical, will use the author’s own provider and key otherwise, and can be switched off entirely." />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="why-h">
        <div className="wrap split">
          <div data-reveal>
            <p className="eyebrow">Why the line</p>
            <h2 id="why-h">What an editor does, and what a writer does.</h2>
          </div>
          <div className="prose" data-reveal>
            <p>
              A good editor remembers what you said on page 40, notices when page 312 forgets it, and asks a pointed question.
              They don’t take the pen. That is the job Asten wants.
            </p>
            <p>
              Language models are genuinely good at parts of that job: finding names, comparing claims, summarizing evidence. They
              are also prone to smoothing every voice toward the same average, and to stating guesses with confidence. A tool that
              writes for you inherits both. A tool that only <em>proposes</em>, with evidence, and lets you decide, inherits only the
              first.
            </p>
            <p>
              Ancient Egyptian scribes kept two inks: black for the text, red to mark it. Thoth, the god of writing for whom
              Asten is named, recorded the verdict at the weighing of the heart. He didn’t decide it. Neither will we.
            </p>
          </div>
        </div>
      </section>

      <section className="register register--raised" aria-labelledby="today-h">
        <div className="wrap">
          <SectionHead
            id="today-h"
            eyebrow="Today"
            title="No model runs inside Asten."
            text="Every red mark you see today comes from a deterministic rule with a stable code. Same project, same marks, every time, with no network involved."
          />
          <div className="card-grid card-grid--3">
            {[
              { title: "Deterministic", text: "Rules you can read and test, from CANON-E013 to CANON-E602, and perception that is dictionaries and cues, not a model." },
              { title: "Offline", text: "Everything runs on your machine. Nothing is sent anywhere." },
              { title: "Explainable", text: "Every mark points to evidence: the settled fact and the page that disagrees." },
            ].map((item, i) => (
              <div key={item.title} className="card" data-reveal style={{ "--d": i } as CSSProperties}>
                <h3>{item.title}</h3>
                <p>{item.text}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="rules-h">
        <div className="wrap">
          <SectionHead
            id="rules-h"
            eyebrow="Tomorrow"
            title="Seven rules for any AI we add."
            text="These are commitments, written down before the features exist, so they shape the features rather than the other way round."
          />
          <ol className="rules">
            {rules.map((rule, i) => (
              <li key={rule.title} data-reveal style={{ "--d": i % 3 } as CSSProperties}>
                <span className="rule-num" aria-hidden="true">
                  {Array.from({ length: i + 1 }, (_, k) => (
                    <i key={k} />
                  ))}
                </span>
                <h3>{rule.title}</h3>
                <p>{rule.text}</p>
              </li>
            ))}
          </ol>
          <p style={{ marginTop: 28 }}>
            <Link to="/ai-policy" className="link-arrow">
              Read the full AI policy <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="aifaq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="aifaq-h" eyebrow="Questions" title="About AI and Asten" />
          <FaqList items={faqs} />
        </div>
      </section>

      <Related paths={["/journal/ai-editor-not-writer", "/ai-policy", "/for/ai-tinkerers", "/features/compiler"]} />
      <CtaBand title="An editor that never takes the pen." />
    </>
  );
}
