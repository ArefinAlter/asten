import { creditedWorks } from "../personality/plates";
import { legalIndex } from "./legal-index";
import type { LegalDoc } from "./types";

/** "Home", or the paths a work appears on; a long list is shortened. */
function appearsOn(pages: string[]): string {
  const names = pages.map((p) => (p === "/" ? "Home" : `\`${p}\``));
  return names.length <= 4 ? names.join(", ") : `${names.slice(0, 3).join(", ")} and ${names.length - 3} more pages`;
}

/** The table of painted works on /credits, made from the same manifest that makes the pictures. */
function artworkRows(): string[][] {
  return creditedWorks().map(({ work, pages }) => {
    const href = work.href.replace(/\(/g, "%28").replace(/\)/g, "%29");
    const medium = work.medium ? `, ${work.medium}` : "";
    return [`[*${work.title}*](${href}), ${work.by}, ${work.date}${medium}`, work.where, work.licence, appearsOn(pages)];
  });
}

export const legalUpdated = "2026-10-25";

export const legalDocs: LegalDoc[] = [
  // ---------------------------------------------------------------------------
  // 1. Privacy
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/privacy"]!,
    summary: [
      "In the desktop app, your manuscript stays in a SQLite file on your computer. Verne never receives it, and Asten has no account, no sync, and no telemetry.",
      "The desktop app has an optional assistant, off by default. If you switch it on, choose a provider and consent, each message you send goes from your computer straight to that provider and nowhere else. See What the assistant sends.",
      "This website sets no cookies and runs no analytics or ads. Vercel, the host, processes ordinary request data to deliver pages and keep the site safe.",
      "The hosted pre-alpha, if you open it, keeps what you type in a private sandbox on a server Verne runs, under one session cookie, and deletes it all 24 hours after your first visit.",
      "If you email Verne, your address and message are used to reply, then deleted within 24 months unless the conversation continues.",
      "Verne does not sell personal information, does not run a mailing list, and will tell you in the interface before any feature sends data anywhere.",
    ],
    sections: [
      {
        id: "who-is-responsible",
        heading: "Who is responsible",
        blocks: [
          {
            kind: "p",
            text: "This policy explains how personal information is handled when you use Asten, visit this website, or write to Verne. Asten and this website are published by Verne Ltd. (“Verne”), registered in Bangladesh under company registration C-201411-1, at 8/A, Sher-e-Bangla Road, Zighatola, Dhaka-1209, Bangladesh.",
          },
          {
            kind: "p",
            text: "For this website and for emails you send, Verne is the controller: the organization that decides why and how that information is used. For a manuscript in the desktop app, Verne is neither a controller nor a processor, because it never reaches Verne. For what you type into the hosted pre-alpha, Verne is the controller for the 24 hours it is kept.",
          },
          {
            kind: "p",
            text: "Verne has not appointed an EU or UK representative. You can reach Verne about anything in this policy at [info@vernesystems.com](mailto:info@vernesystems.com). Company details are published at [vernesystems.com/legal/company](https://vernesystems.com/legal/company).",
          },
        ],
      },
      {
        id: "the-assistant",
        heading: "What the assistant sends",
        blocks: [
          {
            kind: "p",
            text: "Release 0.0.1b has an optional assistant in the desktop app. It is off by default, and with it off the desktop app sends nothing to any AI service. The hosted pre-alpha and the browser desk have no such feature and call no provider.",
          },
          {
            kind: "p",
            text: "**When.** Only when you press send in the assistant panel, and only after you have turned on AI providers, enabled one provider, entered your own key (for a cloud provider), and consented to that provider. Consent is tied to the provider’s address: if the address changes, Asten asks again. A cap of zero means no call, and offline mode allows only a model on your own machine.",
          },
          {
            kind: "p",
            text: "**To whom.** Only to the provider you chose, directly from your computer over an encrypted connection (plain HTTP only to a model on your own machine). The built-in choices are OpenAI (api.openai.com), Anthropic (api.anthropic.com), Google Gemini (generativelanguage.googleapis.com), Mistral (api.mistral.ai), DeepSeek (api.deepseek.com), Qwen (dashscope-intl.aliyuncs.com), Kimi (api.moonshot.ai), GLM (open.bigmodel.cn), Ollama on your machine (127.0.0.1), or an OpenAI-compatible address you enter. Nothing goes to Verne, and Asten refuses redirects so a key cannot follow one to another site.",
          },
          {
            kind: "p",
            text: "**What.** Your question and the conversation so far, the assistant’s fixed instructions, the names of the tools it may use, and what those tools return when it uses them: a mark’s code and explanation, settled facts, cast entries, page titles, source summaries. Your manuscript’s wording is left out unless you pick a passage to share, and wording quoted inside a mark is left out unless you attach that mark. Your key is never part of what is sent to a model; it travels only as the provider’s authentication header.",
          },
          {
            kind: "p",
            text: "**See it first.** Before sending, you can open the exact request, with your key shown as a placeholder. Every call is also written to a spend ledger on your computer (time, provider, model, token counts, cost estimate), which Verne never sees.",
          },
          {
            kind: "p",
            text: "**Your key.** Asten keeps it in your operating system’s credential store (Windows Credential Manager, the Secret Service, the Keychain) or, where there is none, in an encrypted file you unlock with a passphrase. The page you see never receives it back, and it does not appear in logs, errors, exports or the ledger.",
          },
          {
            kind: "callout",
            tone: "note",
            title: "What the provider does with it",
            text: "Once a request reaches a provider you chose, that provider’s own terms and privacy policy govern it. Verne is not a party to that and cannot see it. Read the provider’s terms before you switch it on, and use a provider whose terms you accept for the passages you choose to share.",
          },
        ],
      },
      {
        id: "the-desk",
        heading: "Your manuscript and the desk",
        blocks: [
          {
            kind: "p",
            text: "The desk keeps your projects, scenes, notes, facts, comments, snapshots, tracked changes, and diagnostics in a SQLite database on the computer where Asten runs. In the desktop app, that file is `asten.db` in your operating system’s application-data folder. If you run the local API yourself, it uses `data/asten.db` unless you point `ASTEN_DATABASE_PATH` somewhere else.",
          },
          {
            kind: "p",
            text: "Asten has no account, no sync, and no telemetry. The desktop app and the local API do not upload your manuscript, your world notes, or your diagnostics to Verne or to anyone else. Verne never receives a manuscript you keep there, so Verne cannot read it, share it, or train anything on it. The hosted pre-alpha is different: what you type there is kept on Verne’s server for up to 24 hours (see [the hosted pre-alpha](#hosted-pre-alpha)), and it is never used to train anything either.",
          },
          {
            kind: "p",
            text: "If you copy the database, export a file, or take your work into Word or Google Docs, that copy goes where you send it and is governed by the terms of that place.",
          },
          {
            kind: "callout",
            tone: "note",
            title: "Deleting or exporting your work",
            text: "Because Verne holds no copy, Verne cannot delete or export your manuscript for you. Delete a project in the desk, or delete the database file. Export it from the desk whenever you like. Both are entirely under your control.",
          },
        ],
      },
      {
        id: "the-website",
        heading: "This website",
        blocks: [
          {
            kind: "p",
            text: "This website describes Asten. It has no sign-in and stores no projects.",
          },
          {
            kind: "list",
            items: [
              "**No cookies.** The site sets none. See the [cookie notice](/cookies).",
              "**No analytics and no ads.** There are no tracking scripts, pixels, or advertising partners.",
              "**Self-hosted fonts.** Typefaces are served from this site, so loading a page sends no request to a font service.",
              "**One optional preference.** If you use the light/dark toggle, your browser saves the choice in a `localStorage` key named `asten-theme`. It stays in your browser and is never sent to Verne or anyone else.",
            ],
          },
          { kind: "h3", text: "Hosting by Vercel" },
          {
            kind: "p",
            text: "The site is hosted by Vercel Inc. When your browser asks for a page, Vercel’s servers process ordinary request data: your IP address, your browser’s user agent, the time, the path requested, and the referring page if your browser sends one. Vercel uses this to deliver pages and to protect the site against abuse, such as attacks and floods of automated traffic.",
          },
          {
            kind: "p",
            text: "Vercel handles that data under its own terms and privacy policy, including its own retention periods. Verne adds no tracking on top of it and does not use it to profile visitors.",
          },
        ],
      },
      {
        id: "hosted-pre-alpha",
        heading: "The hosted pre-alpha",
        blocks: [
          {
            kind: "p",
            text: "The hosted pre-alpha at [asten.103.191.50.32.sslip.io](https://asten.103.191.50.32.sslip.io/) lets you try the Asten workbench without installing anything. Unlike the desktop app, it runs on a server that Verne rents and operates, so what you type there does reach Verne’s server. Use it with the sample novel or with text you do not mind leaving your computer; keep your manuscript in the desktop app.",
          },
          {
            kind: "list",
            items: [
              "**A private sandbox.** Your first visit creates a sandbox of your own: a separate project database that no other visitor can see. It holds the projects, documents, facts and settings you create there.",
              "**One cookie.** The sandbox is tied to a strictly necessary session cookie, `asten_visitor`: a random value, `HttpOnly`, `Secure`, `SameSite=Strict`, lasting 24 hours. The server stores only a hash of it. See the [cookie notice](/cookies).",
              "**Deleted after 24 hours.** The sandbox, its settings and the cookie’s hash are deleted 24 hours after your first visit, or sooner if you choose About, Start over. There are no backups of sandboxes.",
              "**Provider keys stay in your tab.** If you type an AI provider key into Settings, it is held in that browser tab’s memory only. It is never sent to the server, and no provider is called in this release.",
              "**Request data.** The server’s web logs record your IP address, the time, the path requested and your browser’s user agent, to keep the service working and to limit abuse. They are rotated and kept no longer than 14 days. Your IP address is also used, for one hour, to limit how many sandboxes one address can create.",
              "**No analytics.** The hosted copy runs no analytics, advertising or tracking scripts, and loads nothing from other sites.",
            ],
          },
          {
            kind: "p",
            text: "The host name ends in `sslip.io`, a public DNS service that turns the name into the server’s address. Your DNS resolver asks it for that address; it receives no cookies and nothing you type.",
          },
        ],
      },
      {
        id: "email",
        heading: "When you email Verne",
        blocks: [
          {
            kind: "p",
            text: "If you write to [info@vernesystems.com](mailto:info@vernesystems.com), Verne receives your email address, your name if you include it, your message and any attachments, and the technical details that travel with every email, such as the date and the sending server.",
          },
          {
            kind: "p",
            text: "Verne uses this to read and answer your message. Verne does not add you to a mailing list, does not send you marketing you did not ask for, and never sells or rents your address.",
          },
          {
            kind: "p",
            text: "Verne keeps your correspondence for as long as it needs to handle your request, then deletes it within 24 months of your last message, unless the conversation continues. Messages are stored with Verne’s business email provider, which handles them on Verne’s behalf; see [subprocessors](/subprocessors).",
          },
          {
            kind: "p",
            text: "Please do not email your manuscript. If you need to show Verne a problem, a short passage is almost always enough.",
          },
        ],
      },
      {
        id: "add-ins",
        heading: "The Word and Google Docs add-ins",
        blocks: [
          {
            kind: "p",
            text: "The Word task pane and the Google Docs sidebar are optional, and you install them yourself by sideloading. Both talk only to the Asten local API at `127.0.0.1:4317` on your own computer. A passage you file is written into your local project. It is not sent to Verne.",
          },
          {
            kind: "p",
            text: "The Word task pane loads Office.js from Microsoft, because Word requires that library. That request is between your computer and Microsoft, under Microsoft’s terms.",
          },
          {
            kind: "p",
            text: "The Docs sidebar runs in Google Apps Script, inside Google’s own service, next to a document Google already holds for you. Asten makes no extra copy of that document on any Verne server. Google’s terms govern Docs and Apps Script.",
          },
        ],
      },
      {
        id: "legal-bases",
        heading: "Legal bases under the GDPR and UK GDPR",
        blocks: [
          {
            kind: "p",
            text: "If the EU General Data Protection Regulation or the UK GDPR applies to you, these are the legal bases Verne relies on:",
          },
          {
            kind: "table",
            head: ["Activity", "Information", "Legal basis"],
            rows: [
              [
                "Delivering this website and keeping it secure",
                "Request data processed by Vercel: IP address, user agent, time, path, referrer",
                "Legitimate interests in running a working, secure website",
              ],
              [
                "Replying to your email",
                "Your address, your message, and anything you include",
                "Your consent; steps you ask Verne to take before an agreement; or Verne’s legitimate interest in answering correspondence",
              ],
              [
                "Running the hosted pre-alpha you open",
                "The `asten_visitor` cookie’s hash, what you create in your sandbox, and web-server request data",
                "Providing the service you asked for, and legitimate interests in keeping it secure",
              ],
              [
                "Remembering your theme",
                "The `asten-theme` value in your browser",
                "None needed: it never leaves your browser, so Verne does not process it",
              ],
            ],
          },
          {
            kind: "p",
            text: "Where Verne relies on legitimate interests, it has weighed them against your rights, and you can object at any time. Where Verne relies on consent, you can withdraw it at any time by writing to Verne. Withdrawing does not affect anything that happened before.",
          },
        ],
      },
      {
        id: "international-transfers",
        heading: "International transfers",
        blocks: [
          {
            kind: "p",
            text: "Verne is based in Bangladesh. If you email Verne from another country, your message travels to Verne in Bangladesh and is stored by Verne’s business email provider, which may process it in other countries too.",
          },
          {
            kind: "p",
            text: "Vercel runs a global network, so request data from your visit may be processed in a country other than yours. Vercel’s own terms describe how it protects data it transfers between countries.",
          },
          {
            kind: "p",
            text: "Bangladesh and some of those countries may not have data protection laws equivalent to those where you live. Verne keeps what it receives to what this policy describes and handles it as described here.",
          },
        ],
      },
      {
        id: "your-rights",
        heading: "Your rights",
        blocks: [
          {
            kind: "p",
            text: "Depending on where you live, you may have the right to:",
          },
          {
            kind: "list",
            items: [
              "access the personal information Verne holds about you;",
              "have it corrected if it is wrong;",
              "have it deleted;",
              "object to processing based on legitimate interests;",
              "restrict processing while a question about it is resolved;",
              "receive it in a portable format;",
              "withdraw consent, where processing is based on consent; and",
              "complain to a data protection supervisory authority, for example in the country where you live or work.",
            ],
          },
          {
            kind: "p",
            text: "To use any of these rights, write to [info@vernesystems.com](mailto:info@vernesystems.com). Verne may ask you to confirm the request is yours, for example by replying from the address you originally wrote from. Verne will respond within the time the applicable law allows.",
          },
          {
            kind: "p",
            text: "In practice, the personal information Verne holds is email correspondence. There is no manuscript on Verne’s side to access, export, or delete; that stays with you. For questions about request data from a visit to this site, write to Verne and Verne will help as far as it can, or contact Vercel directly.",
          },
          { kind: "h3", text: "United States privacy rights" },
          {
            kind: "p",
            text: "If you live in a US state with a consumer privacy law, such as California, you may have the right to know, access, correct, and delete personal information, and to opt out of its sale or sharing. Verne does not sell personal information. Verne does not share it for cross-context behavioral advertising. Verne does not collect sensitive personal information. Verne will not treat you differently for exercising your rights. You, or an agent you authorize, can send requests to [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
      {
        id: "children",
        heading: "Children",
        blocks: [
          {
            kind: "p",
            text: "Asten and this website are not directed at children under 16, and Verne does not knowingly collect personal information from them. If you believe a child has written to Verne, let Verne know and the correspondence will be deleted.",
          },
        ],
      },
      {
        id: "security",
        heading: "Security",
        blocks: [
          {
            kind: "p",
            text: "Your manuscript lives on your computer, so its safety is mostly in your hands. Protect your device and your backups the way you would any private file. The Asten local API binds to the loopback address, `127.0.0.1`, by default, so other computers on your network cannot reach it unless you change that setting.",
          },
          {
            kind: "p",
            text: "This website is served over HTTPS, and its Content Security Policy allows scripts, styles, and fonts only from the site itself. No system is perfectly secure. If you find a weakness, please report it as described on the [security page](/security).",
          },
        ],
      },
      {
        id: "changes",
        heading: "Changes to this policy",
        blocks: [
          {
            kind: "p",
            text: "The “last updated” date on this page shows when this policy last changed. If Verne changes how personal information is handled, this page will be updated before the change takes effect.",
          },
          {
            kind: "p",
            text: "Apart from the Office.js request the Word task pane must make, the update check when you press it, and the assistant’s requests to a provider you chose and consented to (see [what the assistant sends](#the-assistant)), Asten sends nothing off your computer. If a future feature would send any other data anywhere, the product will say so in the interface before that feature sends anything. Updating this page alone will never be treated as enough. See the [AI policy](/ai-policy).",
          },
        ],
      },
      {
        id: "contact",
        heading: "Contact",
        blocks: [
          {
            kind: "p",
            text: "Questions, requests, and complaints about privacy: [info@vernesystems.com](mailto:info@vernesystems.com). By post: Verne Ltd., 8/A, Sher-e-Bangla Road, Zighatola, Dhaka-1209, Bangladesh. The [legal notice](/notice) has full company details.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 2. Terms
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/terms"]!,
    summary: [
      "You may use this website and run the Asten builds Verne gives you for your own writing.",
      "You own everything you write. Verne claims no rights in it, and Asten never trains on it.",
      "Asten is pre-alpha software, provided as is. Features can change, the hosted copy deletes everything after 24 hours, and a diagnostic is a notice, not a verdict.",
      "These terms are governed by the laws of Bangladesh, but the consumer protections where you live still apply.",
    ],
    sections: [
      {
        id: "about-these-terms",
        heading: "About these terms",
        blocks: [
          {
            kind: "p",
            text: "These terms apply to this website and to any Asten build that Verne Ltd. (“Verne”), gives you. By using the site or running a build, you agree to them. If you do not agree, please do not use the site or the builds.",
          },
          {
            kind: "p",
            text: "Two other pages form part of these terms: the [acceptable use policy](/acceptable-use), which says what you may not do on this website and the hosted pre-alpha, and the [privacy policy](/privacy), which explains how personal information is handled. The Asten software itself is released under the MIT License; the [license page](/license) has its text.",
          },
          {
            kind: "p",
            text: "If you use Asten for an organization, you confirm that you are allowed to accept these terms on its behalf.",
          },
        ],
      },
      {
        id: "what-asten-is",
        heading: "What Asten is",
        blocks: [
          {
            kind: "p",
            text: "Asten is a local-first writing environment. The desk stores your project in a SQLite file on your computer, and a rule engine marks contradictions in your story’s canon, such as Mira’s hair turning black after chapter 2 settled it as auburn. Asten does not write prose, and it does not run an AI model.",
          },
          {
            kind: "p",
            text: "This website describes Asten. The hosted pre-alpha keeps a sandbox for 24 hours so you can try Asten; it is not an offer to host your work. Describing a planned feature is not a promise to build it. The [roadmap](/roadmap) says what is planned; only what ships is real.",
          },
        ],
      },
      {
        id: "pre-alpha",
        heading: "Pre-alpha",
        blocks: [
          {
            kind: "p",
            text: "Asten is in early development. Release 0.0.1b is a public pre-alpha: anyone may download the desktop builds or open the hosted copy.",
          },
          {
            kind: "list",
            items: [
              "Builds and the hosted copy are provided as is and as available.",
              "Features may be added, changed, or removed, and a pre-alpha build may stop working or be withdrawn.",
              "The hosted copy deletes each sandbox 24 hours after the first visit, keeps no backups, and may be reset or taken offline at any time. Do not keep work there.",
              "There is no service level, uptime promise, or support commitment.",
              "Keep your own backups. Asten keeps snapshots and a history, but a copy on another disk is still the best protection a manuscript can have.",
            ],
          },
        ],
      },
      {
        id: "your-writing",
        heading: "Your writing",
        blocks: [
          {
            kind: "p",
            text: "You own what you write in Asten: manuscripts, notes, facts, comments, and exports. Verne claims no rights in any of it. These terms give Verne no license to your work, and none is needed, because your work never reaches Verne.",
          },
          {
            kind: "p",
            text: "Asten never trains any model on your writing, and Verne will never use it to train one. See the [AI policy](/ai-policy).",
          },
          {
            kind: "p",
            text: "You are responsible for having the right to hold and use what you import. Importing a file does not change who owns it.",
          },
        ],
      },
      {
        id: "diagnostics",
        heading: "Diagnostics are notices, not verdicts",
        blocks: [
          {
            kind: "p",
            text: "A diagnostic, such as a mark that Captain Oren gives an order after his settled death in chapter 9, points at a passage and names the rule and the fact it disagrees with. It is a notice for you to weigh. Asten checks only the facts you have confirmed, so it can miss things, and it can flag things you meant on purpose.",
          },
          {
            kind: "p",
            text: "You decide what to change. You can suppress any rule. You remain the author of your work and the person responsible for what you publish.",
          },
        ],
      },
      {
        id: "acceptable-use",
        heading: "Acceptable use",
        blocks: [
          {
            kind: "p",
            text: "Use the site and Asten lawfully and in line with the [acceptable use policy](/acceptable-use). In short: work with texts you have a right to hold, leave other people’s computers alone, and do not attack or overload this website.",
          },
        ],
      },
      {
        id: "intellectual-property",
        heading: "Verne’s intellectual property",
        blocks: [
          {
            kind: "p",
            text: "Verne owns the Asten software, the names “Asten” and “Verne”, the Palette mark, the text of this website, and its original artwork. These terms do not transfer any of that to you.",
          },
          {
            kind: "p",
            text: "The Asten software is released under the [MIT License](/license). The [trademark page](/trademark) explains how you may refer to the names and use the logo. The [credits](/credits) list the third-party fonts and software, which keep their own licenses.",
          },
        ],
      },
      {
        id: "feedback",
        heading: "Feedback",
        blocks: [
          {
            kind: "p",
            text: "If you send Verne ideas, suggestions, or bug reports, Verne may use them to improve Asten without owing you anything, and without any obligation to use them. Please do not send material you need kept confidential as feedback, and please do not send a manuscript.",
          },
        ],
      },
      {
        id: "third-party-services",
        heading: "Third-party services",
        blocks: [
          {
            kind: "p",
            text: "Some parts of Asten work alongside services Verne does not control: Microsoft Word and Office.js for the Word task pane, Google Docs and Apps Script for the Docs sidebar, any AI provider you connect to the assistant with your own key, and Vercel, which hosts this website. Your use of those services is governed by their own terms and privacy policies, and Verne is not responsible for them.",
          },
        ],
      },
      {
        id: "disclaimers",
        heading: "Disclaimers",
        blocks: [
          {
            kind: "p",
            text: "To the extent the law allows, this website and Asten are provided “as is” and “as available”, without warranties of any kind, express or implied, including warranties of merchantability, fitness for a particular purpose, accuracy, non-infringement, and uninterrupted or error-free operation.",
          },
          {
            kind: "p",
            text: "In particular, Verne does not promise that Asten will find every contradiction, that every diagnostic is correct, or that a build will never lose data.",
          },
        ],
      },
      {
        id: "limitation-of-liability",
        heading: "Limitation of liability",
        blocks: [
          {
            kind: "p",
            text: "To the extent permitted by law, Verne is not liable for any indirect, incidental, special, or consequential loss arising from your use of this website or Asten, including the loss of a draft that existed only on your computer.",
          },
          {
            kind: "p",
            text: "To the extent permitted by law, Verne’s total liability for all claims relating to this website or Asten is limited to USD 100.",
          },
          {
            kind: "p",
            text: "Nothing in these terms limits or excludes any liability that cannot be limited or excluded by law. If you use Asten as a consumer, nothing in these terms affects the protections you have under the consumer laws of the place where you live.",
          },
        ],
      },
      {
        id: "governing-law",
        heading: "Governing law",
        blocks: [
          {
            kind: "p",
            text: "These terms are governed by the laws of Bangladesh, and the courts of Dhaka have jurisdiction over any dispute about them.",
          },
          {
            kind: "p",
            text: "If you are a consumer, the mandatory consumer laws of the country where you live still apply to you, and this choice of law does not take those protections away.",
          },
          {
            kind: "p",
            text: "Before anything formal, please write to Verne. Most problems are easier to solve in an email.",
          },
        ],
      },
      {
        id: "changes",
        heading: "Changes to these terms",
        blocks: [
          {
            kind: "p",
            text: "Verne may update these terms as Asten develops. The “last updated” date on this page shows the current version. Changes apply from that date and do not reach back to earlier use. If you keep using the site or a build after a change, the updated terms apply; if you do not accept them, stop using the site and the builds.",
          },
        ],
      },
      {
        id: "general",
        heading: "General",
        blocks: [
          {
            kind: "p",
            text: "If a court finds part of these terms unenforceable, the rest still applies. If Verne does not enforce a term straight away, it has not given up the right to enforce it later. You may not transfer your rights under these terms to someone else without Verne’s written agreement.",
          },
        ],
      },
      {
        id: "contact",
        heading: "Contact",
        blocks: [
          {
            kind: "p",
            text: "Questions about these terms: [info@vernesystems.com](mailto:info@vernesystems.com). The [legal notice](/notice) has Verne’s registered address and company details.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 3. Cookies
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/cookies"]!,
    summary: [
      "This website sets no cookies and uses no analytics, advertising, or tracking technology.",
      "The hosted pre-alpha, a separate site, sets one strictly necessary session cookie, `asten_visitor`, for 24 hours.",
      "If you switch between the light and dark themes, your browser remembers the choice in one local setting. It never leaves your browser.",
      "There is no consent banner, because nothing here needs your consent.",
    ],
    sections: [
      {
        id: "no-cookies",
        heading: "No cookies",
        blocks: [
          {
            kind: "p",
            text: "This website does not set any cookies, its own or anyone else’s. It loads no analytics, advertising, social media, or tracking scripts, and its fonts are served from the site itself rather than from a font service.",
          },
          {
            kind: "p",
            text: "The Asten desktop app does not use cookies either. There is no sign-in. Your project is a SQLite file on your computer, which is not a cookie and is not sent to Verne.",
          },
        ],
      },
      {
        id: "hosted-pre-alpha",
        heading: "The hosted pre-alpha’s one cookie",
        blocks: [
          {
            kind: "p",
            text: "The hosted pre-alpha at [asten.103.191.50.32.sslip.io](https://asten.103.191.50.32.sslip.io/) is a separate site from this one. It needs one cookie to know which private sandbox is yours. It stores nothing in your browser’s `localStorage`; your settings are kept with the sandbox on the server.",
          },
          {
            kind: "table",
            head: ["Name", "Purpose", "Set by", "How long", "Sent to anyone?"],
            rows: [
              [
                "`asten_visitor`",
                "Ties your browser to your private sandbox (strictly necessary)",
                "The hosted pre-alpha, `HttpOnly`, `Secure`, `SameSite=Strict`",
                "24 hours from your first visit",
                "Only to that site. The server keeps a hash of it and deletes it with the sandbox.",
              ],
            ],
          },
        ],
      },
      {
        id: "theme-preference",
        heading: "The one thing stored: your theme",
        blocks: [
          {
            kind: "p",
            text: "The site has a light/dark toggle. If you use it, your browser saves the choice in `localStorage`, a small storage area that belongs to your browser. Unlike a cookie, it is not sent to the server with your requests.",
          },
          {
            kind: "table",
            head: ["Name", "Purpose", "Where it is stored", "How long", "Sent to anyone?"],
            rows: [
              [
                "`asten-theme`",
                "Remembers whether you chose the light or the dark theme",
                "Your browser’s `localStorage` for this site",
                "Until you clear it",
                "No. It never leaves your browser.",
              ],
            ],
          },
          {
            kind: "p",
            text: "If you never use the toggle, nothing is stored. The value holds a theme name and nothing that identifies you.",
          },
        ],
      },
      {
        id: "hosting",
        heading: "Hosting infrastructure",
        blocks: [
          {
            kind: "p",
            text: "The site is hosted by Vercel. Like any host, Vercel may rely on strictly necessary infrastructure mechanisms to deliver pages and protect the site against abuse. For example, if the site is under attack, Vercel may briefly show a browser check that stores a short-lived security token, used only to tell visitors apart from automated traffic.",
          },
          {
            kind: "p",
            text: "These mechanisms are not used for analytics, advertising, or tracking you across sites. The [privacy policy](/privacy#the-website) explains what request data Vercel processes.",
          },
        ],
      },
      {
        id: "no-consent-banner",
        heading: "Why there is no consent banner",
        blocks: [
          {
            kind: "p",
            text: "Consent banners exist to ask permission for cookies and trackers that are not strictly necessary. This site sets none, the hosted pre-alpha’s one cookie is strictly necessary for the sandbox you opened, and any security mechanism used by the host is strictly necessary. The theme setting is stored only because you asked for it by pressing the toggle, and it never leaves your browser. There is nothing to consent to, so there is no banner to dismiss.",
          },
        ],
      },
      {
        id: "clearing-it",
        heading: "How to clear it",
        blocks: [
          {
            kind: "list",
            items: [
              "Clear site data for this website in your browser’s privacy or site settings. This removes `asten-theme` and anything else stored for the site.",
              "Or open your browser’s developer tools, find Local Storage for this site, and delete the `asten-theme` entry.",
              "Private or incognito windows discard it when you close them.",
            ],
          },
        ],
      },
      {
        id: "changes",
        heading: "Changes",
        blocks: [
          {
            kind: "p",
            text: "If this site ever needs a cookie or another stored value, this page will name it, say who sets it and how long it lasts, and be updated before it is used. Questions: [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 4. License
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/license"]!,
    summary: [
      "Asten is open source under the MIT License: you may use, copy, change, and redistribute it, including commercially.",
      "Keep the copyright and license notice with any copy you pass on.",
      "Your manuscripts are yours. The license gives Verne no rights in them.",
      "The MIT License does not cover the Asten and Verne names or logos.",
    ],
    sections: [
      {
        id: "mit-license",
        heading: "The MIT License",
        blocks: [
          {
            kind: "p",
            text: "The Asten source code on [GitHub](https://github.com/ArefinAlter/asten) and the builds Verne publishes, including the desk, the desktop app, the local API, and the Word and Google Docs add-ins, are released under the MIT License. Its full text, which is also the LICENSE file in the repository, follows.",
          },
          {
            kind: "p",
            text: "Copyright (c) 2026 Verne Ltd.",
          },
          {
            kind: "p",
            text: "Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:",
          },
          {
            kind: "p",
            text: "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.",
          },
          {
            kind: "p",
            text: "THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.",
          },
        ],
      },
      {
        id: "your-work",
        heading: "Your work is not licensed to Verne",
        blocks: [
          {
            kind: "p",
            text: "Your manuscripts, notes, facts, comments, snapshots, exports, and the local database are yours. The license runs one way, from Verne to you. You grant Verne nothing in your writing, and Asten never trains on it.",
          },
          {
            kind: "p",
            text: "You may keep, copy, export, publish, and sell what you write with Asten, without credit or payment to Verne.",
          },
        ],
      },
      {
        id: "names-and-logos",
        heading: "Names and logos",
        blocks: [
          {
            kind: "p",
            text: "The MIT License covers the software, not Verne’s trademarks. You may say that your work is based on Asten, but a modified version or a hosted service must not present itself as Asten or as a Verne product. See the [trademark page](/trademark).",
          },
        ],
      },
      {
        id: "pre-alpha-builds",
        heading: "Pre-alpha builds",
        blocks: [
          {
            kind: "p",
            text: "Asten is in early development, and release 0.0.1b is a pre-alpha. A build may stop running, may change between versions, and may be replaced or withdrawn. Verne has no obligation to provide updates, support, or any particular feature.",
          },
          {
            kind: "p",
            text: "Your project file does not depend on a build. Before a build is replaced or withdrawn, keep the database, or export your work, and you will lose nothing.",
          },
        ],
      },
      {
        id: "third-party-components",
        heading: "Third-party components",
        blocks: [
          {
            kind: "p",
            text: "Asten and this website include components made by others, such as fonts and open-source libraries. Each keeps its own license, and nothing in this license limits the rights those licenses give you. The [credits](/credits) list them.",
          },
        ],
      },
      {
        id: "relationship-to-terms",
        heading: "How this fits with the terms",
        blocks: [
          {
            kind: "p",
            text: "The MIT License governs the software. The [terms of use](/terms) and the [acceptable use policy](/acceptable-use) govern this website and the hosted pre-alpha that Verne runs. Questions: [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 5. Acceptable use
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/acceptable-use"]!,
    summary: [
      "Use Asten with texts you have the right to hold and use.",
      "Do not use Asten’s local API to reach a computer you do not control, and do not attack or overload this website.",
      "Do not impersonate Verne or send false notices.",
    ],
    sections: [
      {
        id: "scope",
        heading: "Who this applies to",
        blocks: [
          {
            kind: "p",
            text: "This policy applies to anyone who uses this website or the hosted pre-alpha that Verne runs. It is part of the [terms of use](/terms), and breaking it can end your access to the hosted pre-alpha. Copies of Asten you run yourself are governed by the [MIT License](/license).",
          },
        ],
      },
      {
        id: "your-texts",
        heading: "Your texts",
        blocks: [
          {
            kind: "p",
            text: "Import, file, index, and compile only works you have the right to hold and use: your own writing, works you have permission to use, and material the law where you are lets you use, such as public-domain texts.",
          },
          {
            kind: "p",
            text: "Asten does not check this for you. Calling a file a research note or a source does not change who owns it.",
          },
        ],
      },
      {
        id: "other-computers",
        heading: "Other people’s computers",
        blocks: [
          {
            kind: "p",
            text: "The Asten local API listens on `127.0.0.1:4317`, the loopback address, so that only programs on your own computer can reach it. Do not:",
          },
          {
            kind: "list",
            items: [
              "scan for, connect to, read, or change an Asten project or local API on a computer you do not control, without its owner’s permission;",
              "trick anyone into exposing their local API, or into installing a modified build or add-in; or",
              "use an Asten add-in to read or file text from a document you are not allowed to access.",
            ],
          },
          {
            kind: "p",
            text: "If you change the address your own local API listens on, you are responsible for who can reach it.",
          },
        ],
      },
      {
        id: "this-website",
        heading: "This website",
        blocks: [
          {
            kind: "list",
            items: [
              "Do not probe, scan, or test the site for vulnerabilities, except for good-faith research under the [security policy](/security).",
              "Do not overload the site with automated requests or try to disrupt it.",
              "Do not try to get around its security measures or reach anything not meant to be public.",
            ],
          },
        ],
      },
      {
        id: "honesty",
        heading: "Names and notices",
        blocks: [
          {
            kind: "list",
            items: [
              "Do not impersonate Verne, or claim to speak for Verne or Asten.",
              "Do not suggest that Verne endorses you or your work; see the [trademark page](/trademark).",
              "Do not send false or misleading copyright, trademark, or security notices.",
            ],
          },
        ],
      },
      {
        id: "research-tools",
        heading: "Research tools",
        blocks: [
          {
            kind: "p",
            text: "Verne also builds separate research tools. They are not part of Asten, and they are not a license to scrape websites, get around paywalls or access controls, or copy or redistribute copyrighted books. If you use them, follow the law and each source’s own terms.",
          },
        ],
      },
      {
        id: "enforcement",
        heading: "If this policy is broken",
        blocks: [
          {
            kind: "p",
            text: "Verne may refuse or end your access to builds, block abusive traffic to this website, or report unlawful activity to the authorities. Verne cannot reach into your computer, and it cannot switch off a database on your disk.",
          },
          {
            kind: "p",
            text: "To report misuse, write to [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 6. AI policy
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/ai-policy"]!,
    summary: [
      "Asten is an AI product designed as an intelligent continuity editor. Its canon engine checks your story against facts you confirm, while the AI assistant explains, searches and audits.",
      "The assistant supports local models and cloud providers via your own key or direct subscriptions (Claude Code, Codex, Antigravity, Cursor, Grok). It strictly hands creativity back to the author: it never writes or ghostwrites your prose.",
      "What is sent, when, and to whom is stated below. Your manuscript is never used to train a model, by Verne or by any provider Verne chooses.",
      "Asten will tell you in the interface before this policy changes.",
    ],
    sections: [
      {
        id: "today",
        heading: "Asten today",
        blocks: [
          {
            kind: "p",
            text: "Asten’s checks do not run any AI model, on your computer or in the cloud. When Asten marks that Mira’s hair is black in chapter 14 after chapter 2 settled it as auburn, that mark comes from a deterministic rule comparing facts you confirmed. The same project gives the same result every time.",
          },
          {
            kind: "p",
            text: "Release 0.0.1b adds an optional assistant to the desktop app. The assistant panel answers from Asten’s own tools by default. If you turn on AI providers, choose one, and consent, it can also call that provider’s model to explain a mark, search your canon, summarise a source, or propose a fact for you to confirm. It has ten tools that read or prepare and no tool that edits your manuscript; a request to write, rewrite or continue prose is refused before any provider is contacted, and a reply that reads like new narrative is withheld. [What the assistant sends](/privacy#the-assistant) lists exactly what leaves your computer, when, and to whom.",
          },
          {
            kind: "p",
            text: "Asten does not generate prose, and it never will. The tagline is a design rule: **let AI be your editor, not your writer.** Your words are black ink; Asten only ever writes in red.",
          },
        ],
      },
      {
        id: "principles",
        heading: "Principles every AI feature meets",
        blocks: [
          {
            kind: "p",
            text: "The assistant in release 0.0.1b is the first optional assisted feature. Further ones, such as proposing facts from your prose for you to confirm, are plans, not promises, and none has a date; see the [roadmap](/roadmap). Every AI feature follows these principles:",
          },
          {
            kind: "list",
            items: [
              "**Editor, not writer.** AI in Asten never writes, rewrites, or continues your prose.",
              "**Proposals only.** AI output arrives as a proposal: a suggested fact, a possible contradiction, a question. Nothing takes effect until you accept it.",
              "**Never silent canon.** An AI proposal never becomes a confirmed fact on its own. Only you can confirm it; the assistant has no tool that does.",
              "**Local first.** Deterministic rules come first, then models that run on your computer (through Ollama or any OpenAI-compatible address on your machine). Nothing leaves your computer unless you decide it should.",
              "**Your provider, your key, your consent.** Asten sends text to a cloud AI provider only if you choose that provider, use your own account and key, and consent to that provider. Consent is tied to the provider’s address, and the interface shows which provider a request goes to.",
              "**A complete switch-off.** You can turn every AI feature off in one place (Settings, AI providers, Allow AI providers). Off means no model runs and nothing is sent. Offline mode allows only a model on your own machine, and a spending cap of zero means no call.",
              "**No training on manuscripts, ever.** Your manuscript will never be used to train or improve a model, by Verne or by any provider Verne chooses.",
              "**You can see what is sent.** Before a request leaves, you can open it exactly as the provider will receive it, with your key replaced by a placeholder. A local ledger records each call: when, which provider and model, how many tokens, and an estimated cost.",
            ],
          },
          {
            kind: "callout",
            tone: "note",
            title: "If you bring your own provider",
            text: "When you connect a provider with your own account, that provider’s terms govern what it does with the text you send. Read them before you turn the feature on.",
          },
        ],
      },
      {
        id: "how-verne-builds-asten",
        heading: "How Verne builds Asten",
        blocks: [
          {
            kind: "p",
            text: "Verne may use AI tools in its own work, such as writing code or drafting text. That is outside this policy, with one firm exception: your manuscripts are never part of it. Verne never receives your manuscript, so it cannot be fed into any tool Verne uses.",
          },
        ],
      },
      {
        id: "changes",
        heading: "How this policy changes",
        blocks: [
          {
            kind: "p",
            text: "If this policy changes, Asten will tell you in the interface first, and this page will show the new date. Consent you gave is tied to a version of the notice: a new version asks again. Turning on any AI feature is always your choice.",
          },
          {
            kind: "p",
            text: "For the thinking behind this policy, see [Asten and AI](/ai) and [AI as editor, not writer](/journal/ai-editor-not-writer). Questions: [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 7. Accessibility
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/accessibility"]!,
    summary: [
      "Verne aims for this website to meet WCAG 2.2 at level AA.",
      "Pages use real headings, work with a keyboard, show focus clearly, and respect reduced-motion settings.",
      "There has been no independent audit yet. If something blocks you, email Verne; the aim is to reply within 10 business days.",
    ],
    sections: [
      {
        id: "our-aim",
        heading: "The aim",
        blocks: [
          {
            kind: "p",
            text: "Anyone should be able to read about Asten. For this website, Verne’s target is the Web Content Accessibility Guidelines (WCAG) 2.2 at level AA. This statement covers the website and describes the goals for the desk.",
          },
        ],
      },
      {
        id: "measures",
        heading: "What has been done",
        blocks: [
          {
            kind: "list",
            items: [
              "**Structure.** Each page has one main heading and real subheadings in order, so you can move through a page by heading. Pages declare English as their language.",
              "**Keyboard.** Navigation, links, the theme toggle, and the interactive examples can be used with a keyboard alone.",
              "**Visible focus.** Whatever has keyboard focus is clearly outlined.",
              "**Reduced motion.** When your system asks for reduced motion, animations stop or are kept to a minimum.",
              "**Contrast.** Text and controls are designed to meet AA contrast in both the light and the dark theme.",
              "**Text alternatives.** Meaningful images have text alternatives. Purely decorative artwork is hidden from assistive technology.",
              "**Not color alone.** Color never carries meaning by itself. Asten’s red-ink marks always come with words or a code.",
            ],
          },
        ],
      },
      {
        id: "known-limitations",
        heading: "Known limitations",
        blocks: [
          {
            kind: "list",
            items: [
              "**Decorative animation.** Some pages animate Egyptian motifs when reduced motion is not requested. The animations carry no information, but some visitors may find them distracting; turning on reduced motion in your system settings stops them.",
              "**Examples are illustrations.** The interactive examples show how Asten behaves, in simplified form. They may not convey everything to a screen reader as well as plain text does. The same ideas are explained in prose on [How it works](/how-it-works) and the [feature pages](/features).",
              "**No independent audit.** The site has been checked by Verne, not by an independent auditor, so this statement is a self-assessment.",
            ],
          },
        ],
      },
      {
        id: "the-desk",
        heading: "The desk",
        blocks: [
          {
            kind: "p",
            text: "The desk is a separate application in early development. Verne’s goals for it are full keyboard operation, a sensible focus order, standard controls that work with screen readers, readable contrast in both themes, and diagnostics that use words and codes, never color alone. Verne does not yet claim a conformance level for the desk.",
          },
        ],
      },
      {
        id: "feedback",
        heading: "Tell Verne about a problem",
        blocks: [
          {
            kind: "p",
            text: "If something on this website or in Asten gets in your way, write to [info@vernesystems.com](mailto:info@vernesystems.com) with “Accessibility” in the subject. Please include:",
          },
          {
            kind: "list",
            items: [
              "the page address, or the part of Asten you were using;",
              "what you were trying to do, and what happened instead; and",
              "if you are comfortable sharing it, your browser, device, and any assistive technology you use.",
            ],
          },
          {
            kind: "p",
            text: "Verne aims to reply within 10 business days.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 8. Copyright
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/copyright"]!,
    summary: [
      "You own your writing. Asten and Verne claim no copyright in it.",
      "Verne owns the text and the original artwork on this website.",
      "If you believe something on this site infringes your copyright, email Verne with the details listed below.",
    ],
    sections: [
      {
        id: "your-writing",
        heading: "Your writing",
        blocks: [
          {
            kind: "p",
            text: "Copyright in your manuscripts, notes, and exports stays with you, or with whoever already owned it. A diagnostic that points at your sentence, a fact you confirm, or a passage you file through an add-in gives Verne no claim on your work. Verne never receives it.",
          },
        ],
      },
      {
        id: "verne-content",
        heading: "Verne’s content",
        blocks: [
          {
            kind: "p",
            text: "The text, design, illustrations, and logos on this website, and the Asten software, belong to Verne Ltd., except where the [credits](/credits) say otherwise. All rights are reserved.",
          },
          {
            kind: "p",
            text: "You may quote short passages with a link back, as the law on quotation and fair use allows. For logos and press assets, see the [brand page](/brand) and the [trademark page](/trademark).",
          },
        ],
      },
      {
        id: "egyptian-artwork",
        heading: "The Egyptian artwork",
        blocks: [
          {
            kind: "p",
            text: "The ornaments, icons and diagrams on this site are original drawings made by Verne, inspired by Egyptian art of the Old, Middle, and New Kingdoms, which is public-domain heritage. The painted plates are public-domain works and CC0 Open Access images from museum collections; Verne cut, graded and arranged them, and claims no rights in the works themselves.",
          },
          {
            kind: "p",
            text: "Verne’s copyright covers its own drawings, not the ancient works or the motifs themselves. Anyone is free to draw on the same heritage.",
          },
        ],
      },
      {
        id: "notices",
        heading: "Reporting infringement",
        blocks: [
          {
            kind: "p",
            text: "This website hosts no uploads from visitors, so a notice will usually concern Verne’s own text or artwork. Asten keeps manuscripts only on its users’ computers, which Verne cannot see or change.",
          },
          {
            kind: "p",
            text: "If you believe material on this site infringes your copyright, email [info@vernesystems.com](mailto:info@vernesystems.com) with “Copyright notice” in the subject, and include:",
          },
          {
            kind: "list",
            ordered: true,
            items: [
              "the URL of the page, and a description of the material;",
              "the copyrighted work you say is infringed, or a representative list if there are several;",
              "your name, postal address, and email address, and, if you act for someone else, whom you represent;",
              "a statement that you believe in good faith that the use is not authorized by the copyright owner, its agent, or the law;",
              "a statement that the information in your notice is accurate and, under penalty of perjury, that you are the owner or are authorized to act for the owner; and",
              "your physical or electronic signature. Typing your full name is enough.",
            ],
          },
          {
            kind: "p",
            text: "Verne will review each notice and, where it is valid, remove or change the material and tell you what was done.",
          },
        ],
      },
      {
        id: "counter-notices",
        heading: "Counter-notices",
        blocks: [
          {
            kind: "p",
            text: "If material was removed after a notice and you believe that was a mistake, for example because you wrote the quoted text or hold a license to it, email [info@vernesystems.com](mailto:info@vernesystems.com) with “Counter-notice” in the subject. Include the URL, why you believe the removal was a mistake, your contact details, a statement made in good faith, and your signature. Verne will review it and may restore the material.",
          },
        ],
      },
      {
        id: "repeat-infringers",
        heading: "Repeat infringers",
        blocks: [
          {
            kind: "p",
            text: "Repeat-infringer policies apply to services that host user content. This website hosts none, and Asten keeps manuscripts on your own computer, so there are no accounts to suspend. If that ever changes, this page will explain how repeat infringement is handled before it does.",
          },
          {
            kind: "p",
            text: "Please do not send false notices. Knowingly misrepresenting infringement can make you liable, and it breaks the [acceptable use policy](/acceptable-use).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 9. Trademark
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/trademark"]!,
    summary: [
      "“Asten”, the Palette mark, and “Verne” are Verne’s trademarks.",
      "You can refer to Asten accurately, use the press assets from the brand page unaltered, and say your work was written with Asten.",
      "Do not imply that Verne endorses you, put the marks in your own product names or domains, or alter the logo.",
    ],
    sections: [
      {
        id: "marks",
        heading: "Verne’s marks",
        blocks: [
          {
            kind: "p",
            text: "These are trademarks of Verne Ltd.:",
          },
          {
            kind: "list",
            items: [
              "the name **Asten**;",
              "the **Palette mark**, Asten’s logo; and",
              "the name **Verne** and Verne’s logo.",
            ],
          },
          {
            kind: "p",
            text: "This page explains how you may use them without asking. It does not grant any license beyond what it describes.",
          },
        ],
      },
      {
        id: "what-is-fine",
        heading: "What is fine without asking",
        blocks: [
          {
            kind: "list",
            items: [
              "Referring to Asten by name in reviews, articles, tutorials, videos, and conversation, as long as what you say is accurate.",
              "Saying that your book, script, or story was written with Asten.",
              "Using the logos and screenshots from the [brand page](/brand), unaltered, in press coverage and articles about Asten.",
              "Linking to this website using the name Asten.",
            ],
          },
        ],
      },
      {
        id: "what-needs-permission",
        heading: "What needs permission",
        blocks: [
          {
            kind: "list",
            items: [
              "Suggesting that Verne sponsors, endorses, or is connected with you, your product, or your work.",
              "Using “Asten”, the Palette mark, or “Verne”, or anything confusingly similar, in the name of your product, company, app, add-in, domain name, or social media account.",
              "Changing the logo: recoloring, stretching, cropping, animating, adding to it, or combining it with another mark.",
              "Using the marks on merchandise, or in advertising for anything other than Asten.",
              "Using the Egyptian artwork from this site as your own branding.",
            ],
          },
        ],
      },
      {
        id: "writing-the-name",
        heading: "Writing the name",
        blocks: [
          {
            kind: "p",
            text: "Write “Asten” with a capital A and the rest in lowercase. It is a name, not a verb: your novel was written with Asten; it was not “Astened”.",
          },
        ],
      },
      {
        id: "requests",
        heading: "Asking for permission",
        blocks: [
          {
            kind: "p",
            text: "For anything this page does not cover, write to [info@vernesystems.com](mailto:info@vernesystems.com) with “Trademark” in the subject. Say what you want to do and where. Permission is given only in writing.",
          },
          {
            kind: "p",
            text: "If you see the marks being misused, Verne would like to hear about it at the same address.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 10. Security
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/security"]!,
    summary: [
      "Report security problems to info@vernesystems.com with “Security” in the subject.",
      "Verne will acknowledge your report within 5 business days and asks for 90 days before public disclosure.",
      "Good-faith research within this policy is welcome, and Verne will not pursue legal action over it.",
      "There is no bug bounty at present.",
    ],
    sections: [
      {
        id: "how-to-report",
        heading: "How to report",
        blocks: [
          {
            kind: "p",
            text: "Email [info@vernesystems.com](mailto:info@vernesystems.com) with “Security” in the subject line. Please include:",
          },
          {
            kind: "list",
            items: [
              "a description of the problem and its likely impact;",
              "the steps to reproduce it, with any proof-of-concept code or screenshots;",
              "the affected URL, or the affected part of Asten with its version or build date and your operating system; and",
              "whether, and how, you would like to be credited.",
            ],
          },
          {
            kind: "p",
            text: "Please do not include anyone else’s personal data or manuscript. If you need to send something sensitive, say so in your first message, and Verne will reply with how to send it.",
          },
        ],
      },
      {
        id: "in-scope",
        heading: "In scope",
        blocks: [
          {
            kind: "list",
            items: [
              "This website.",
              "Asten builds distributed by Verne, including the desk and the desktop app.",
              "The Asten local API.",
              "The Word task pane and the Google Docs sidebar.",
            ],
          },
        ],
      },
      {
        id: "out-of-scope",
        heading: "Out of scope",
        blocks: [
          {
            kind: "list",
            items: [
              "Social engineering, phishing, or physical attacks against Verne or its people.",
              "Denial-of-service attacks, or any test that degrades the website for others.",
              "Services run by third parties, such as Vercel, Microsoft, and Google. Please report those to them.",
            ],
          },
        ],
      },
      {
        id: "local-api",
        heading: "About the local API",
        blocks: [
          {
            kind: "p",
            text: "The Asten local API binds to the loopback address, `127.0.0.1:4317`, by default, so only software on the same computer can reach it. It can be configured to listen elsewhere; if you do that, put it behind your own access controls. Problems with how the API behaves on loopback, including how it treats requests from web pages open in your browser, are in scope.",
          },
        ],
      },
      {
        id: "safe-harbor",
        heading: "Safe harbor",
        blocks: [
          {
            kind: "p",
            text: "If you research and report in good faith under this policy, Verne will treat your research as authorized, will not pursue or support legal action against you for it, and will work with you to understand and fix the problem. Good faith means you:",
          },
          {
            kind: "list",
            items: [
              "test only against your own installation and data, or systems you are authorized to test;",
              "avoid privacy violations, destruction of data, and interruption of service;",
              "access only what you need to show the problem, and stop and report as soon as you have;",
              "do not disclose the problem before Verne has had a reasonable chance to fix it; and",
              "follow the law.",
            ],
          },
          {
            kind: "p",
            text: "This safe harbor covers Verne’s own claims. It cannot bind third parties such as Vercel, Microsoft, or Google. If you are unsure whether something is allowed, ask first.",
          },
        ],
      },
      {
        id: "disclosure",
        heading: "What happens next",
        blocks: [
          {
            kind: "p",
            text: "Verne will acknowledge your report within 5 business days, keep you informed, and tell you when the problem is fixed.",
          },
          {
            kind: "p",
            text: "Please give Verne 90 days from your report before disclosing it publicly, so a fix can reach people first. If a fix needs longer, Verne will explain why and agree a date with you. If a fix ships sooner, Verne is happy to coordinate an earlier disclosure.",
          },
        ],
      },
      {
        id: "rewards",
        heading: "Rewards",
        blocks: [
          {
            kind: "p",
            text: "Verne does not run a bug bounty and does not currently pay for reports. Your help is appreciated all the same.",
          },
        ],
      },
      {
        id: "security-txt",
        heading: "Machine-readable contact",
        blocks: [
          {
            kind: "p",
            text: "The contact details in this policy are also published at `/.well-known/security.txt` on this site, in the standard security.txt format.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 11. Subprocessors
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/subprocessors"]!,
    summary: [
      "No third party processes your manuscript for Verne, because Verne never receives it.",
      "Vercel hosts this website. A server Verne rents runs the hosted pre-alpha. Verne’s business email provider carries the messages you send to Verne.",
      "Microsoft and Google are involved only if you choose to use the Word or Google Docs add-ins.",
      "Verne uses no analytics, advertising, payment, or customer-relationship vendors.",
    ],
    sections: [
      {
        id: "the-desk",
        heading: "The desk",
        blocks: [
          {
            kind: "p",
            text: "None. Your manuscript is stored in SQLite on your computer and is never sent to Verne, so no vendor processes it on Verne’s behalf. The optional assistant in release 0.0.1b can send requests to an AI provider you choose with your own key and consent; that provider acts for you, under your own agreement with it, and Verne receives nothing through it. If a future build would send data to any other third party, the interface will say so first, and this page will be updated.",
          },
        ],
      },
      {
        id: "website-and-email",
        heading: "Website and email",
        blocks: [
          {
            kind: "table",
            head: ["Name", "Purpose", "Data", "Location"],
            rows: [
              [
                "Vercel Inc.",
                "Hosting and edge delivery of this website, and protection against abuse",
                "Request data: IP address, user agent, time, path, referrer",
                "United States company with a global edge network",
              ],
              [
                "The server provider for the hosted pre-alpha",
                "A virtual server Verne rents and operates, which runs the hosted pre-alpha and its sandboxes",
                "What you create in a sandbox (deleted after 24 hours), the `asten_visitor` cookie’s hash, web-server request data",
                "May be outside your country; see [international transfers](/privacy#international-transfers)",
              ],
              [
                "Verne’s business email provider",
                "Receiving, storing, and sending mail for info@vernesystems.com",
                "Your email address, name, message, attachments, and email metadata",
                "May be outside your country; see [international transfers](/privacy#international-transfers)",
              ],
            ],
          },
          {
            kind: "p",
            text: "Each works under its own terms and data protection commitments. The [privacy policy](/privacy) explains what is collected and why.",
          },
        ],
      },
      {
        id: "only-if-you-choose",
        heading: "Only if you choose them",
        blocks: [
          {
            kind: "table",
            head: ["Name", "Purpose", "Data", "Location"],
            rows: [
              [
                "Microsoft",
                "Supplies Office.js, which the Word task pane must load, and runs Word itself",
                "Ordinary request data for Office.js; your Word document, under your own agreement with Microsoft",
                "Microsoft’s infrastructure",
              ],
              [
                "Google",
                "Runs the Docs sidebar in Google Apps Script, and Google Docs itself",
                "Your document and Apps Script usage, under your own agreement with Google",
                "Google’s infrastructure",
              ],
            ],
          },
          {
            kind: "p",
            text: "These are not Verne’s subprocessors in the usual sense. You use them under your own agreements with Microsoft and Google, and Verne receives nothing through them. The add-ins send the passages you file only to the Asten local API on your computer.",
          },
        ],
      },
      {
        id: "not-used",
        heading: "What Verne does not use",
        blocks: [
          {
            kind: "p",
            text: "No analytics, advertising, payment, or customer-relationship (CRM) vendors. No font services, tag managers, or social media widgets. This website’s Content Security Policy allows scripts, styles, and fonts only from the site itself.",
          },
        ],
      },
      {
        id: "changes",
        heading: "Changes",
        blocks: [
          {
            kind: "p",
            text: "If Verne adds a service that processes personal information, this page will be updated before that service starts. Questions: [info@vernesystems.com](mailto:info@vernesystems.com).",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 12. Credits
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/credits"]!,
    summary: [
      "This website is set in Fraunces, Literata, IBM Plex Mono, Caveat, and Noto Sans Egyptian Hieroglyphs, all under the SIL Open Font License 1.1.",
      "It is built with React, React Router, Vite, and TypeScript. Its social images are rendered at build time with Satori and resvg-js.",
      "The painted plates are public-domain or CC0 works from museum and library collections, credited in the table below. The ornaments and icons are original drawings by Verne.",
    ],
    sections: [
      {
        id: "fonts",
        heading: "Fonts",
        blocks: [
          {
            kind: "table",
            head: ["Font", "Designer and copyright", "License"],
            rows: [
              [
                "Fraunces",
                "Undercase Type. © 2020 The Fraunces Project Authors",
                "SIL Open Font License 1.1",
              ],
              [
                "Literata",
                "TypeTogether for Google. © 2017 The Literata Project Authors",
                "SIL Open Font License 1.1",
              ],
              [
                "IBM Plex Mono",
                "IBM. © 2017 IBM Corp.",
                "SIL Open Font License 1.1",
              ],
              [
                "Caveat",
                "Impallari Type, for Google Fonts. © 2014 The Caveat Project Authors",
                "SIL Open Font License 1.1",
              ],
              [
                "Noto Sans Egyptian Hieroglyphs",
                "Google. © 2022 The Noto Project Authors",
                "SIL Open Font License 1.1",
              ],
            ],
          },
          {
            kind: "p",
            text: "The font files are served from this website, packaged by the Fontsource project. The SIL Open Font License allows fonts to be used, studied, modified, and redistributed freely, as long as they are not sold on their own. Its full text is at [openfontlicense.org](https://openfontlicense.org).",
          },
        ],
      },
      {
        id: "software",
        heading: "Software",
        blocks: [
          {
            kind: "table",
            head: ["Software", "Copyright", "License"],
            rows: [
              ["React and React DOM", "Meta Platforms, Inc. and affiliates", "MIT"],
              [
                "React Router",
                "React Training LLC, Remix Software Inc., and Shopify Inc.",
                "MIT",
              ],
              ["Vite", "VoidZero Inc. and Vite contributors", "MIT"],
              ["TypeScript", "Microsoft Corporation", "Apache License 2.0"],
              ["Satori", "Vercel", "Mozilla Public License 2.0"],
              ["resvg-js", "The resvg-js contributors", "Mozilla Public License 2.0"],
            ],
          },
          {
            kind: "p",
            text: "Satori and resvg-js run only at build time, to render the images shown when a page is shared. They are not sent to your browser.",
          },
          {
            kind: "p",
            text: "License texts: [MIT](https://opensource.org/license/mit), [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0), and [MPL 2.0](https://www.mozilla.org/MPL/2.0/). Each package also ships its own copy.",
          },
        ],
      },
      {
        id: "in-asten",
        heading: "In Asten builds",
        blocks: [
          {
            kind: "p",
            text: "The desk is built on open-source work too. The main components include SQLite (public domain), Tauri (MIT or Apache 2.0), Axum and Tokio (MIT), SQLx (MIT or Apache 2.0), Tiptap and ProseMirror (MIT), and React (MIT). The assistant adds rustls and ring (Apache 2.0, MIT and ISC-style), reqwest (MIT or Apache 2.0), keyring (MIT or Apache 2.0), and the RustCrypto implementations of Argon2 and ChaCha20-Poly1305 (MIT or Apache 2.0). Importing and exporting Word and Open Packaging files uses quick-xml and zip (MIT). This is not a complete list, and each component’s own license applies.",
          },
        ],
      },
      {
        id: "artwork",
        heading: "Artwork",
        blocks: [
          {
            kind: "p",
            text: "The painted plates on this site are works in the public domain, or images that their holders release under Creative Commons Zero: the Metropolitan Museum of Art (Open Access), the National Gallery of Art, the Cleveland Museum of Art, the Yale Center for British Art and the British Library. The others are old enough that no right remains in them, and the libraries and museums that scanned them make no claim over a faithful photograph. Each is cropped, cut from its backdrop where it is an object, graded to one warm light and set on the paper by Verne; nothing is redrawn or generated. The table gives the holder’s own title, date and object number, and the build that makes the files (art/build.py, with art/works.json and art/plates.json) is in the source repository.",
          },
          {
            kind: "table",
            head: ["Work", "Holding institution", "Licence", "Appears on"],
            rows: artworkRows(),
          },
          {
            kind: "p",
            text: "The ornaments, icons, friezes and diagrams are original SVG drawings by Verne. See [copyright](/copyright) for how everything on this site may be used.",
          },
        ],
      },
      {
        id: "hieroglyphs",
        heading: "Hieroglyphs and the cartouche toy",
        blocks: [
          {
            kind: "p",
            text: "Where this site refers to a hieroglyph by code, the code follows Gardiner’s sign list, the standard catalog from Sir Alan Gardiner’s *Egyptian Grammar*. G26, for example, is the sacred ibis on a standard, the sign of Thoth.",
          },
          {
            kind: "callout",
            tone: "note",
            title: "A game, not a translation",
            text: "The cartouche name toy spells names with a popular, simplified “hieroglyphic alphabet” that matches letters to single signs. It is not real Middle Egyptian. Egyptian writing mixed sound signs with signs for words and meanings, so a scholar would write your name differently.",
          },
        ],
      },
    ],
  },

  // ---------------------------------------------------------------------------
  // 13. Legal notice
  // ---------------------------------------------------------------------------
  {
    ...legalIndex["/notice"]!,
    summary: [
      "Asten and this website are published by Verne Ltd., registered in Dhaka, Bangladesh.",
      "The quickest way to reach Verne is info@vernesystems.com.",
    ],
    sections: [
      {
        id: "publisher",
        heading: "Publisher",
        blocks: [
          {
            kind: "list",
            items: [
              "**Product:** Asten, a local-first writing environment",
              "**Publisher:** Verne Ltd.",
              "**Registered address:** 8/A, Sher-e-Bangla Road, Zighatola, Dhaka-1209, Bangladesh",
              "**Company registration:** C-201411-1",
              "**Email:** [info@vernesystems.com](mailto:info@vernesystems.com)",
              "**Website:** [vernesystems.com](https://vernesystems.com)",
              "**Company details:** [vernesystems.com/legal/company](https://vernesystems.com/legal/company)",
              "**Asten website:** [asten-writing.vercel.app](https://asten-writing.vercel.app)",
            ],
          },
        ],
      },
      {
        id: "responsible-for-content",
        heading: "Responsible for content",
        blocks: [
          {
            kind: "p",
            text: "Verne Ltd., at the address above, is responsible for the content of this website. If you spot something inaccurate or out of date on any page, write to [info@vernesystems.com](mailto:info@vernesystems.com) and Verne will look into it.",
          },
        ],
      },
      {
        id: "dispute-resolution",
        heading: "Dispute resolution",
        blocks: [
          {
            kind: "p",
            text: "Verne is not obliged to take part, and does not take part, in dispute resolution proceedings before a consumer arbitration board.",
          },
          {
            kind: "p",
            text: "That does not mean a complaint goes nowhere. You can write to [info@vernesystems.com](mailto:info@vernesystems.com) at any time, and Verne will try to put things right directly.",
          },
        ],
      },
      {
        id: "hosting",
        heading: "Hosting",
        blocks: [
          {
            kind: "p",
            text: "This website is hosted by Vercel Inc. The Asten desktop app runs on your own computer; your manuscript is not stored on Verne’s servers or on Vercel’s. The hosted pre-alpha runs on a server Verne rents and deletes each visitor’s sandbox after 24 hours.",
          },
        ],
      },
      {
        id: "other-pages",
        heading: "Other legal pages",
        blocks: [
          {
            kind: "p",
            text: "For how personal information is handled, see the [privacy policy](/privacy). For the rules of use, see the [terms of use](/terms) and the [license](/license). This site links to third-party websites; their operators are responsible for their content.",
          },
        ],
      },
    ],
  },
];
