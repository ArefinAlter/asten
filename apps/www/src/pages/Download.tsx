import { useState } from "react";
import { Link } from "react-router-dom";
import { PaletteMark } from "../art/Palette";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Inline } from "../components/Inline";
import { Answer, PageHero, Related, SectionHead } from "../components/Page";
import { accessSteps } from "../content/home";
import type { QA } from "../content/types";
import { accessHref, site } from "../site";

const faqs: QA[] = [
  { q: "Is there a public download of Asten?", a: "Not yet. Asten is in early access, and builds go to invited writers with setup notes. A signed desktop installer is next on the [roadmap](/roadmap)." },
  { q: "How much does Asten cost?", a: "There is nothing to buy today. Pricing hasn’t been decided. Whatever happens, your projects will stay readable and exportable without a subscription or a server." },
  { q: "What do I need on my computer?", a: "Windows 10 or 11 (x64 or ARM), macOS 11 or later (Apple silicon or Intel), or a Linux desktop with WebKitGTK (Ubuntu 22.04 and newer). Asten runs as a desktop shell, or in your browser against a small server on your own machine; phone browser installation requires HTTPS with a certificate trusted by the phone. These are configured platform targets; installers for each system still need verification before distribution." },
  { q: "Is Asten open source?", a: "No. Asten’s source repository is private today. Its rules and diagnostic codes are documented publicly on this site." },
];

function CopyEmail() {
  const [done, setDone] = useState(false);
  return (
    <span className="copy">
      {site.email}
      <button
        type="button"
        className={done ? "is-done" : undefined}
        onClick={async () => {
          try {
            await navigator.clipboard.writeText(site.email);
            setDone(true);
            window.setTimeout(() => setDone(false), 1800);
          } catch {
            window.location.href = accessHref;
          }
        }}
      >
        {done ? "Copied" : "Copy"}
      </button>
    </span>
  );
}

export function Download() {
  return (
    <>
      <PageHero
        eyebrow="Early access"
        title="Get Asten on your own machine."
        lede="Asten is in early access. There’s no public download yet. We’re inviting writers in small groups so we can listen properly. Ask for an invitation and tell us what you write."
        art={<PaletteMark className="hero-palette" />}
      >
        <div className="btn-row">
          <a className="btn btn--primary" href={accessHref} data-magnetic>
            Ask for early access
          </a>
          <CopyEmail />
        </div>
        <p className="fine" style={{ marginTop: 16 }}>
          Subject line: “{site.accessSubject}”. We reply from the same address. No mailing list. See the{" "}
          <Link to="/privacy" className="link">
            privacy policy
          </Link>
          .
        </p>
      </PageHero>

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten is available by invitation during early access. Email info@vernesystems.com with the subject “Asten early access”. Invited writers receive a build and setup notes; Asten then runs entirely on their own computer, with no account." />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="steps-h">
        <div className="wrap">
          <SectionHead id="steps-h" eyebrow="What happens" title="From invitation to first scene." />
          <ol className="steps" data-reveal>
            {accessSteps.map((step, i) => (
              <li key={step.title} className="step" id={`step-${i + 1}`}>
                <span className="step-num" aria-hidden="true">
                  {Array.from({ length: i + 1 }, (_, k) => (
                    <i key={k} />
                  ))}
                </span>
                <h3>{step.title}</h3>
                <p>
                  <Inline text={step.text} />
                </p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="include-h">
        <div className="wrap split">
          <div data-reveal>
            <p className="eyebrow">In the build</p>
            <h2 id="include-h">What early access includes.</h2>
          </div>
          <ul className="honest-list" data-reveal>
            <li>The desk, canon, cast, two clocks, and optional world tools</li>
            <li>Three continuity rules and four method packs</li>
            <li>Compile and import for Markdown, plain text, Fountain, HTML, and .docx</li>
            <li>The Word task pane and Google Docs sidebar, for sideloading</li>
            <li>No account, no sync, no telemetry, and no AI model</li>
          </ul>
        </div>
      </section>

      <section className="register" aria-labelledby="dfaq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="dfaq-h" eyebrow="Questions" title="About early access" />
          <FaqList items={faqs} />
          <p style={{ marginTop: 24 }}>
            <Link to="/roadmap" className="link-arrow">
              See what’s next <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <Related paths={["/how-it-works", "/features/local-first", "/roadmap", "/privacy"]} />
    </>
  );
}
