import type { Article } from "../types";
import { journalMeta } from "./meta";

export const article: Article = {
  ...journalMeta["ai-editor-not-writer"]!,
  answer:
    "Use AI the way you’d use a good editor: to remember what you decided, check new pages against it, ask questions, and point at evidence. Keep the sentences for yourself. Prose drafted by a model drifts toward the average, and the voice, choices, and ownership that make a story yours drift with it.",
  takeaways: [
    "Drafting is where your voice lives; hand it to a model and the book drifts toward the average.",
    "An editor’s most useful work is memory and evidence: what you decided, and where a page disagrees.",
    "That work suits software because you can check it, and because it never has to touch your sentences.",
    "Asten today runs deterministic rules over facts you confirm. No AI model runs in it.",
    "Any future AI in Asten is planned as proposals you confirm, local first, and never trained on your manuscript.",
  ],
  sections: [
    {
      id: "what-goes-wrong",
      heading: "What goes wrong when AI writes the book?",
      blocks: [
        {
          kind: "p",
          text: "You’ve probably tried it. You paste a scene into a chatbot, ask it to “make this better,” and get back something smoother, tidier, and somehow no longer yours. Nothing in it is wrong, but it no longer sounds like you.",
        },
        {
          kind: "p",
          text: "Language models are built to produce a likely next word. That is useful, and it is also why model prose drifts toward the middle: the familiar image, the expected beat, the sentence a thousand other writers might have written. Your voice is mostly the set of unlikely choices you keep making on purpose. The odd comparison. The clipped line. The joke the scene doesn’t strictly need. Smooth those out and you get competent pages that could belong to anyone.",
        },
        {
          kind: "p",
          text: "There’s a quieter cost, too. When a draft is half yours and half a model’s, it gets hard to say which decisions you made. Revision turns strange, because you’re editing a stranger’s instincts. And many writers simply feel that the work stops being theirs in the way that matters to them, whatever the rules about it say. When a reader quotes a line back to you someday, you want to know it was yours.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "This isn’t an argument against AI. It’s an argument about which job you give it.",
        },
      ],
    },
    {
      id: "what-an-editor-does",
      heading: "What does a good editor actually do?",
      blocks: [
        {
          kind: "p",
          text: "Much of an editor’s most useful work is routine. A good editor:",
        },
        {
          kind: "list",
          items: [
            "**Remembers.** That Mira’s hair is auburn, that the harbor at Veld freezes in winter, that Captain Oren died in chapter 9.",
            "**Checks.** Reads the new chapter against everything that came before and notices where it slips.",
            "**Questions.** Asks “Did you mean this?” instead of announcing “This is wrong.”",
            "**Points at evidence.** Shows you the two sentences that disagree, so you can judge for yourself.",
          ],
        },
        {
          kind: "p",
          text: "Notice what isn’t on that list: rewriting your sentences. Line editors do suggest changes, of course, but the good ones treat a suggestion as a question with a pencil attached. The decision stays with the author. An editor who takes over the prose has stopped editing.",
        },
        {
          kind: "table",
          head: ["", "AI as writer", "AI as editor"],
          rows: [
            ["Holds", "The pen", "The record"],
            ["Produces", "Sentences and scenes", "Questions and marks"],
            ["Answers to", "What’s likely", "Evidence on the page"],
            ["When it misses", "The book sounds like someone else", "You dismiss a note"],
          ],
          caption: "Two jobs you could give AI, and what each costs when it gets something wrong.",
        },
        {
          kind: "p",
          text: "Notice, too, how much of the job is memory and comparison. In a long manuscript (a doorstop fantasy, a fan-fiction series that has run for years, a campaign setting with hundreds of named places) that is exactly the part people are worst at and most tired of. It’s also the part a machine can do without going anywhere near your voice.",
        },
      ],
    },
    {
      id: "black-ink-red-ink",
      heading: "Why black ink and red ink?",
      blocks: [
        {
          kind: "p",
          text: "Egyptian scribes carried a palette with two cakes of ink. Black was for the text. Red was for marking it: headings, dates, verse points, corrections. The two never got confused, because they did different jobs.",
        },
        {
          kind: "p",
          text: "Asten takes its [name](/name) from that tradition. It’s another name for Thoth, the scribe of the Egyptian gods, who stood beside the scales at the weighing of the heart and recorded the verdict without deciding it. Asten takes the same role: your text is the black ink, and Asten’s marks are the red.",
        },
        {
          kind: "p",
          text: "In practice the rule is simple. Everything you type is yours and stays untouched. Everything Asten adds (a mark in the margin, a code, a note) sits beside your text, points at a span, and can be dismissed. A [diagnostic](/glossary#diagnostic) never rewrites the sentence it’s about. It looks like this:",
        },
        {
          kind: "diagnostic",
          code: "CANON-E013",
          title: "Mira’s hair was settled as auburn.",
          lines: [
            "① Settled: Mira · hair · auburn",
            "② Chapter 19 says: “black”",
            "Add an event · Retire “auburn” · Silence this mark · Keep my sentence",
          ],
        },
        {
          kind: "p",
          text: "The mark doesn’t say which side is right. Maybe chapter 19 is the slip. Maybe Mira dyed her hair offstage and the story needs to show it. Maybe chapter 2 was wrong all along. That’s your call. The mark only makes sure you get to make it before a reader does.",
        },
      ],
    },
    {
      id: "why-software-suits-editing",
      heading: "Why is editing a better job for software than writing?",
      blocks: [
        {
          kind: "h3",
          text: "You can check it",
        },
        {
          kind: "p",
          text: "When a tool says “these two facts disagree,” you can look at both and know in seconds whether it’s right. When a tool hands you a paragraph, there’s no such check. There’s only taste, and taste was the thing you were supposed to supply.",
        },
        {
          kind: "h3",
          text: "It stays out of the sentences",
        },
        {
          kind: "p",
          text: "A consistency check doesn’t need to touch your prose to be useful. It reads, compares, and points. Your style survives because nobody else is holding the pen.",
        },
        {
          kind: "h3",
          text: "It takes the boring part",
        },
        {
          kind: "p",
          text: "Few people become writers for the bookkeeping. Remembering eye colors, death dates, and which spring the war ended is necessary work that crowds out the work you love. Handing that off costs you nothing you care about.",
        },
        {
          kind: "h3",
          text: "It can be wrong without doing harm",
        },
        {
          kind: "p",
          text: "No checker is perfect. A mark that turns out to be wrong costs you a glance and a click. Sometimes the contradiction is deliberate: a character lies about her past, or a narrator misremembers on purpose. Then you keep your sentence and [suppress](/glossary#suppression) the mark, with a note about why. A model’s paragraph that turns out to be wrong for your book costs far more, because by the time you notice, it has already shaped the scenes around it.",
        },
        {
          kind: "callout",
          tone: "rubric",
          title: "The author decides",
          text: "Asten keeps the record; you keep the pen. Every mark names the rule, the sentence, and the settled fact it disagrees with, and any rule can be silenced with a reason.",
        },
      ],
    },
    {
      id: "what-asten-does-today",
      heading: "What does Asten do today, and what doesn’t it do?",
      blocks: [
        {
          kind: "p",
          text: "Asten is a local-first writing desk in early development. Here is what it does now.",
        },
        {
          kind: "p",
          text: "You write in the [desk](/features/desk): a binder of chapters and scenes, a rich-text editor, comments, and named snapshots. As you go, you settle the details that matter as [canon](/glossary#canon). A [fact](/glossary#fact) is just an entity, a property, and a value, like *Mira · hair · auburn*. The [compiler](/features/compiler) then weighs the draft against that record with fifteen rules in seven families. Three examples:",
        },
        {
          kind: "list",
          items: [
            "**CANON-E013** marks an attribute with two values at the same point in story time, like auburn and black.",
            "**CANON-E101** marks a character who acts after a settled death.",
            "**CANON-E201** marks story time stepping backward inside a run of scenes you declared continuous.",
          ],
        },
        {
          kind: "p",
          text: "Those rules are deterministic. **No AI model runs in Asten today**, local or cloud. The checks compare facts you’ve confirmed with each other and with plain attributes that fixed rules read from the page. They don’t claim to understand your story, and nothing they read from the page becomes canon unless you settle it yourself. Optional craft hints (a note when you lean on “very” or “really”) live in a [method pack](/glossary#method-pack) that is off by default, and they’re labeled as notes, not errors.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "There’s nothing to buy today. Asten is in early development; release 0.0.1a is a public pre-alpha you can try in the browser or download for Windows and Linux. See [Download](/download).",
        },
      ],
    },
    {
      id: "what-asten-plans",
      heading: "What will AI do in Asten later?",
      blocks: [
        {
          kind: "p",
          text: "The [roadmap](/roadmap) has AI on it, in a narrow role. Asten does not make fact proposals today: its fixed rules read the page and weigh it against what you settled. The planned step is small local classifiers that read more of the page and propose facts, along the lines of “This looks like Mira’s hair is auburn. Settle it?” Every proposal will wait for you to confirm it. Nothing becomes canon silently.",
        },
        {
          kind: "list",
          items: [
            "**Local models first.** A cloud model only with your own provider and key, and only with your explicit consent.",
            "**A complete switch-off.** You can turn AI off entirely.",
            "**No training on manuscripts.** Your book is never used to train any model.",
            "**No prose.** Asten won’t produce your sentences, now or later.",
          ],
        },
        {
          kind: "p",
          text: "The models are planned, not shipped, and there are no dates. The [AI page](/ai) and the [AI policy](/ai-policy) say the same thing at more length.",
        },
      ],
    },
    {
      id: "editor-habits",
      heading: "How can you use AI as an editor with the tools you already have?",
      blocks: [
        {
          kind: "p",
          text: "You don’t need Asten to take this stance. If you keep a chatbot open beside your draft, a few habits keep you on the editor side of the line:",
        },
        {
          kind: "list",
          ordered: true,
          items: [
            "**Ask for questions, not rewrites.** “What would a careful reader find confusing in this scene?” beats “Improve this scene.”",
            "**Ask for evidence.** “Quote the lines where the timeline might conflict” gives you something you can check.",
            "**Keep the record yourself.** A plain list of settled facts (names, ages, deaths, dates) is worth more than any clever prompt.",
            "**Mind where your pages go.** Read a tool’s data and training terms before you paste in a chapter.",
            "**Write the sentence yourself, even when the suggestion is good.** Take the idea; leave the wording.",
          ],
        },
        {
          kind: "p",
          text: "The point isn’t purity. It’s that the book should come out sounding like you, and that you should be able to say, without hedging, that you wrote it.",
        },
      ],
    },
  ],
  faqs: [
    {
      q: "Does Asten use AI to check my story?",
      a: "No. No AI model runs in Asten today, local or cloud. Its checks are deterministic rules that compare facts you have confirmed.",
    },
    {
      q: "Will Asten ever draft prose?",
      a: "No. Asten will not produce an author’s prose. Planned AI features are fact proposals that you confirm, never sentences in your manuscript.",
    },
    {
      q: "Is my manuscript used to train AI models?",
      a: "No. Asten’s principles rule out using any manuscript to train a model, and your project stays on your computer, so Verne never receives it.",
    },
    {
      q: "What is the difference between an AI writer and an AI editor?",
      a: "An AI writer produces the sentences. An AI editor remembers what you decided, checks new pages against it, and points at evidence, leaving every word and every decision to you.",
    },
  ],
  related: [
    "/ai",
    "/features/compiler",
    "/features/canon",
    "/journal/continuity-in-a-long-novel",
    "/roadmap",
  ],
};
