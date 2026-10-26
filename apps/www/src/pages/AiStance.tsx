import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Answer, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import type { QA } from "../content/types";

const rules = [
  { title: "Editor, never writer", text: "No feature will generate, continue, rewrite, or autocomplete your prose. This applies to every feature, including experimental ones." },
  { title: "Proposals only", text: "AI may propose a fact, a link, or an explanation. Nothing it suggests becomes canon until you confirm it." },
  { title: "Evidence attached", text: "Every proposal cites the sentence it came from, so you can check it." },
  { title: "Local first", text: "Where a model is practical on your own machine, that is the default." },
  { title: "Your provider and key", text: "If you choose a cloud model, you choose the provider, you hold the key (kept in your operating system’s credential store, never in the page), and you consent to that provider before anything is sent." },
  { title: "You can read the request", text: "Before a request leaves, Asten shows it exactly as the provider will receive it, with your key replaced by a placeholder. Your manuscript’s wording is left out unless you pick a passage." },
  { title: "A full switch-off", text: "AI features can be switched off completely. Asten remains a full desk and a full compiler without them." },
  { title: "No training on manuscripts", text: "Your manuscript is never used to train any model, by Verne or anyone Verne works with." },
];

const faqs: QA[] = [
  { q: "Does Asten use AI?", a: "Only if you ask it to. Asten’s continuity checks run no model: they are deterministic rules over facts you confirm. Release 0.0.1b adds an optional assistant to the desktop app that can use a model you choose. It is off by default, local models come first, a cloud provider needs your own key and your consent, and one switch turns it all off. The [AI policy](/ai-policy) says exactly what is sent, when, and to whom." },
  { q: "Why would a writing app refuse to write?", a: "Because the writing is the part that makes the work yours. Asten aims at the part writers find tedious: remembering hundreds of settled details and noticing when a page drifts from them." },
  { q: "Can I use ChatGPT or Claude with Asten?", a: "In the desktop app, yes, with your own account: the assistant can call OpenAI, Anthropic, Google Gemini, Mistral, DeepSeek, Qwen, Kimi, GLM, a model on your own machine through Ollama, or any OpenAI-compatible endpoint you enter. You choose the provider, you hold the key, you consent to that provider, and you can read the exact request before it is sent. The assistant never writes your prose. What you paste into other tools is governed by their terms." },
  { q: "Does the assistant cost money?", a: "Asten has nothing to buy, and pricing hasn’t been decided. If you use a cloud provider, that provider bills your own account; Asten keeps a local spend ledger and lets you set a cap, and a cap of zero means no call at all. The commitments on this page don’t depend on price." },
];

export function AiStance() {
  return (
    <>
      <PageHero
        title="Asten and AI"
        lede="Asten does not write prose. Its checks run no model. An optional assistant can use a model you choose, under the conditions this page states."
      />

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten never generates prose and never trains on manuscripts. Its checks run no model; they are deterministic rules. The optional assistant in release 0.0.1b explains marks, searches your canon and proposes facts with cited evidence for you to confirm; it runs a model locally where you have one, uses your own provider and key otherwise, only with your consent, and can be switched off entirely." />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="why-h">
        <div className="wrap split">
          <div data-reveal>
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
              Asten is named, recorded the verdict at the weighing of the heart without deciding it. Asten takes the same role: it records and points, and the author decides.
            </p>
          </div>
        </div>
      </section>

      <section className="register register--raised" aria-labelledby="today-h">
        <div className="wrap">
          <SectionHead
            id="today-h"
            title="No model runs inside the checks."
            text="Every mark comes from a deterministic rule with a stable code. The same project produces the same marks, and no network is involved."
          />
          <div className="card-grid card-grid--3">
            {[
              { title: "Deterministic", text: "Rules you can read and test, from CANON-E013 to CANON-E601, and perception that is dictionaries and cues, not a model." },
              { title: "Offline by default", text: "Everything runs on your machine. The only things that leave it are a request you send from the assistant to a provider you chose and consented to, and the update check when you press it." },
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
            title="The conditions every AI feature meets"
            text="These commitments were written down before the features existed, so they shaped the features rather than the other way round."
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
          <SectionHead id="aifaq-h" title="About AI and Asten" />
          <FaqList items={faqs} />
        </div>
      </section>

      <Related paths={["/journal/ai-editor-not-writer", "/ai-policy", "/for/ai-tinkerers", "/features/compiler"]} />
      <CtaBand />
    </>
  );
}
