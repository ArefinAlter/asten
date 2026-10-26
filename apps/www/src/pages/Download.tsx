import { Link } from "react-router-dom";
import { FaqList } from "../components/Blocks";
import { Arrow } from "../components/Icons";
import { Inline } from "../components/Inline";
import { Answer, PageHero, Related, SectionHead } from "../components/Page";
import { accessSteps } from "../content/home";
import { DownloadCards, HeroDownload, UiIcon } from "../components/DownloadCards";
import { release } from "../content/release";
import type { QA } from "../content/types";

const faqs: QA[] = [
  { q: "What is the pre-alpha?", a: "Release 0.0.1a is the first public build of Asten. It is early software: the engine, the desk and compile work, but expect rough edges and changes between releases. Keep your own copies of anything you write in it." },
  { q: "What is the difference between the hosted copy and the desktop app?", a: "Both run the same engine. The desktop app runs it on your computer and keeps your projects in one file on your disk; it is the way to use Asten for real work. The hosted copy runs it on Verne’s server in a private sandbox per visitor, for trying Asten without installing anything. The sandbox and everything in it are deleted 24 hours after your first visit. Do not put your manuscript there." },
  { q: "Why does my system warn me before it runs?", a: "The installers are not code-signed yet. On Windows, SmartScreen shows “Windows protected your PC”; choose More info, then Run anyway. Compare the file’s SHA-256 with its line in SHA256SUMS.txt first (the “Verify downloads” link on this page). Signed installers are next on the [roadmap](/roadmap)." },
  { q: "Is there a macOS build?", a: "Not in 0.0.1a. macOS comes later. On a Mac you can use the hosted copy in the meantime." },
  { q: "How much does Asten cost?", a: "Nothing today. Pricing hasn’t been decided. Whatever happens, your projects will stay readable and exportable without a subscription or a server." },
  { q: "Does it update itself?", a: "No. Settings, This desk, Check for updates asks for a newer release only when you press it, and installs only after a second press. Nothing checks on its own." },
];

export function Download() {
  return (
    <>
      <PageHero
        title="Try the pre-alpha"
        lede="Asten 0.0.1a is out as a pre-alpha. Open it in your browser on a sample novel, or install the desktop app for Windows or Linux and write on your own machine."
      >
        <div className="btn-row">
          <HeroDownload />
          <a className="btn btn--ghost" href={release.hostedUrl}>
            <UiIcon name="globe" />
            Open it in the browser
          </a>
        </div>
        <p className="fine" style={{ marginTop: 16 }}>
          Released {release.dateLabel}. The browser copy keeps a private sandbox for 24 hours and sets one session
          cookie; see the{" "}
          <Link to="/privacy" className="link">
            privacy policy
          </Link>
          .
        </p>
      </PageHero>

      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <div data-reveal>
            <Answer text="Asten 0.0.1a is a pre-alpha. The desktop app for Windows x64 and Linux x64 runs the engine on your own computer, keeps your projects in one file on your disk and offers “Open the sample novel”. The hosted copy runs the same engine on Verne’s server in a private sandbox per visitor, deleted 24 hours after the first visit. The builds are unsigned. macOS comes later." />
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="steps-h">
        <div className="wrap">
          <SectionHead id="steps-h" title="Two ways to try it" />
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

      <section className="register register--sunk" id="downloads" aria-labelledby="downloads-h">
        <div className="wrap">
          <SectionHead id="downloads-h" title={`Release ${release.version} downloads`} />
          <DownloadCards />
          <ul className="honest-list dl-after" data-reveal>
            <li>
              <span>Windows: run the installer (it installs for your user only), or unzip the portable build and run Asten.exe.</span>
            </li>
            <li>
              <span>
                Linux: make the AppImage executable with <code>chmod +x</code> and run it, or install the .deb with{" "}
                <code>sudo apt install ./Asten_0.0.1-alpha_amd64.deb</code>.
              </span>
            </li>
            <li>
              <span>asten-api: run it, open 127.0.0.1:4317 and type the pairing code it prints.</span>
            </li>
          </ul>
        </div>
      </section>

      <section className="register" aria-labelledby="include-h">
        <div className="wrap split">
          <div data-reveal>
            <h2 id="include-h">What 0.0.1a includes.</h2>
          </div>
          <ul className="honest-list" data-reveal>
            <li>The desk, canon, cast, two clocks, and optional world tools</li>
            <li>Fifteen canon rules in seven families, plus name hygiene and a setup ledger, in eight method packs</li>
            <li>Compile to Markdown, plain text, Fountain, HTML, .docx, EPUB 3, and Final Draft (.fdx), and import of all but EPUB</li>
            <li>The Word task pane and Google Docs sidebar, for sideloading against the desktop app</li>
            <li>No account, no sync, no telemetry; AI providers can be configured, but none is called in this release</li>
          </ul>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="dfaq-h">
        <div className="wrap wrap-narrow">
          <SectionHead id="dfaq-h" title="About the pre-alpha" />
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
