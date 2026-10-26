import { Link } from "react-router-dom";
import { Wedjat } from "../art/Figures";
import { Glyph } from "../art/Glyphs";
import { Arrow, External } from "../components/Icons";
import { Answer, CtaBand, PageHero, Related, SectionHead } from "../components/Page";
import { release } from "../content/release";
import { site } from "../site";

const pillars = [
  { title: "The pen stays yours", text: "Asten never generates prose. Anything AI touches in the future is a proposal you confirm." },
  { title: "Every mark shows its evidence", text: "A mark names the rule, the sentence, and the settled fact it disagrees with." },
  { title: "The house is yours", text: "Local-first. No account, no sync, no telemetry, no training on your words." },
  { title: "Quiet craft", text: "No doctrine, no nagging. Packs are optional and craft hints start switched off." },
  { title: "Honest about the work", text: "We say what ships today and what is planned, and we never blur the two." },
];

export function About() {
  return (
    <>
      <PageHero
        title="About Asten and Verne"
        lede="Asten is designed and built by Verne, an AI engineering company in Dhaka, Bangladesh."
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <Answer text={`Asten is designed and built by ${site.legalName}, a company registered in Dhaka, Bangladesh, that builds AI systems under human review. Asten applies the same principle to writing: software can remember, compare, and point at evidence, but the author writes and decides.`} />
        </div>
      </section>
      <section className="register" aria-labelledby="why-built">
        <div className="wrap split">
          <div>
            <h2 id="why-built">Why Asten exists</h2>
          </div>
          <div className="prose">
            <p>
              Writing a long story has two halves. One is remembering: hundreds of small decisions about who has which scar, who
              is dead, what day it is, and which side of the reef the harbor sits on. The other half is everything that makes the
              book yours.
            </p>
            <p>
              Most AI writing tools reach for the second half. We think software belongs in the first. So Asten is a writing desk
              that keeps a record of your story, weighs each new page against it, and marks what doesn’t balance. It never writes
              a sentence.
            </p>
            <p>
              Verne also builds AI systems for other organizations, with evaluation and human review. Asten applies the same
              principle to writing software.
            </p>
          </div>
        </div>
      </section>
      <section className="register register--sunk" aria-labelledby="pillars-h">
        <div className="wrap">
          <SectionHead id="pillars-h" title="Principles" />
          <ol className="rules">
            {pillars.map((pillar, i) => (
              <li key={pillar.title} data-reveal>
                <span className="rule-num" aria-hidden="true">
                  {Array.from({ length: i + 1 }, (_, k) => (
                    <i key={k} />
                  ))}
                </span>
                <h3>{pillar.title}</h3>
                <p>{pillar.text}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>
      <section className="register" aria-labelledby="facts-h">
        <div className="wrap split">
          <div>
            <h2 id="facts-h">Company details</h2>
          </div>
          <div className="table-wrap">
            <table>
              <tbody>
                <tr>
                  <th scope="row">Product</th>
                  <td>Asten, a local-first writing environment</td>
                </tr>
                <tr>
                  <th scope="row">Publisher</th>
                  <td>{site.legalName}</td>
                </tr>
                <tr>
                  <th scope="row">Based in</th>
                  <td>Dhaka, Bangladesh</td>
                </tr>
                <tr>
                  <th scope="row">Status</th>
                  <td>Pre-alpha (release 0.0.1b, Windows and Linux)</td>
                </tr>
                <tr>
                  <th scope="row">Contact</th>
                  <td>
                    <a className="link" href={`mailto:${site.email}`}>
                      {site.email}
                    </a>
                  </td>
                </tr>
                <tr>
                  <th scope="row">Verne</th>
                  <td>
                    <a className="link" href={site.publisherUrl} target="_blank" rel="noopener noreferrer">
                      vernesystems.com
                    </a>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
      <Related paths={["/name", "/ai", "/brand", "/contact"]} />
      <CtaBand />
    </>
  );
}

const channels = [
  { glyph: "scroll" as const, title: "Pre-alpha feedback", text: "What worked, what broke, and which computer you use.", subject: "Asten pre-alpha" },
  { glyph: "mouth" as const, title: "Press", text: "Interviews, review builds, and brand assets beyond the press kit.", subject: "Asten press" },
  { glyph: "door" as const, title: "Partnerships", text: "Writing programs, schools, and publishers.", subject: "Asten partnership" },
  { glyph: "eye" as const, title: "Privacy", text: "Questions about your data, or a request under privacy law.", subject: "Asten privacy" },
  { glyph: "seated" as const, title: "Accessibility", text: "A page or feature that gets in your way. Tell us which and how.", subject: "Asten accessibility" },
  { glyph: "scales" as const, title: "Legal and copyright", text: "Notices, takedowns, trademark questions.", subject: "Asten legal" },
];

export function Contact() {
  return (
    <>
      <PageHero
        title="Contact"
        lede="One address reaches the people who build Asten. Choosing a subject below helps route your message."
      >
        <p>
          <a className="btn btn--primary" href={`mailto:${site.email}`} data-magnetic>
            {site.email}
          </a>
        </p>
      </PageHero>
      <section className="register register--sunk" aria-labelledby="ch-h">
        <div className="wrap">
          <SectionHead id="ch-h" title="Subjects" />
          <div className="card-grid card-grid--3">
            {channels.map((channel) => (
              <a key={channel.title} className="card card--carved" href={`mailto:${site.email}?subject=${encodeURIComponent(channel.subject)}`}>
                <Glyph name={channel.glyph} />
                <h3>{channel.title}</h3>
                <p>{channel.text}</p>
                <span className="card-more">
                  Subject: {channel.subject} <Arrow />
                </span>
              </a>
            ))}
            <Link className="card card--carved" to="/security">
              <Glyph name="house" />
              <h3>Security</h3>
              <p>Found a vulnerability? Read the disclosure policy first, then write with “Security” in the subject.</p>
              <span className="card-more">
                Disclosure policy <Arrow />
              </span>
            </Link>
          </div>
        </div>
      </section>
      <section className="register" aria-labelledby="post-h">
        <div className="wrap split">
          <div>
            <h2 id="post-h">Postal address</h2>
          </div>
          <div className="prose">
            <p>
              {site.legalName}
              <br />
              {site.address}
            </p>
            <p>
              Registration and procurement details are published at{" "}
              <a href={site.companyDetails} target="_blank" rel="noopener noreferrer">
                vernesystems.com/legal/company <External />
              </a>
              . Verne is also on{" "}
              <a href={site.social.x} target="_blank" rel="noopener noreferrer">
                X
              </a>{" "}
              and{" "}
              <a href={site.social.linkedin} target="_blank" rel="noopener noreferrer">
                LinkedIn
              </a>
              .
            </p>
            <p>
              What happens to your email is in the <Link to="/privacy">privacy policy</Link>: we use it to reply, and that’s all.
            </p>
          </div>
        </div>
      </section>
      <section className="register register--sunk" aria-label="Try the pre-alpha">
        <div className="wrap cta-band">
          <Wedjat />
          <div>
            <h2>Try the pre-alpha</h2>
            <p>Open it in your browser, or download release 0.0.1b for Windows or Linux. No form to fill in.</p>
          </div>
          <a className="btn btn--primary" href={release.hostedUrl} data-magnetic>
            Try the pre-alpha
          </a>
        </div>
      </section>
    </>
  );
}
