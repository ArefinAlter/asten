import type { Article } from "../types";
import { journalMeta } from "./meta";

export const article: Article = {
  ...journalMeta["do-you-need-a-story-bible"]!,
  answer:
    "You need a story bible when your world holds more details than you can reliably remember: a series, a shared universe, a system of rules, a long timeline. Keep it lean and in service of the draft. Gardeners can build it from what the pages settle; architects can plan ahead but should stop before planning replaces writing.",
  takeaways: [
    "A story bible is a reference for keeping a world consistent, not a ticket you need before you start writing.",
    "Architects plan first, gardeners discover as they draft, and most writers are some of each.",
    "Worldbuilding can quietly become a way of not writing. Watch for it.",
    "Most of the world should stay below the sand: build what the story shows, and let the rest stay a sketch.",
    "In Asten the world is optional, and each entry moves from spark to sketched to settled as you decide.",
  ],
  sections: [
    {
      id: "what-is-a-story-bible",
      heading: "What is a story bible?",
      blocks: [
        {
          kind: "p",
          text: "A [story bible](/glossary#story-bible) is the reference book for your fictional world: who the characters are, what they look like, where things are, what happened when, and how the world works. Television writers call it a series bible, a document that lets a room full of writers keep a show consistent across seasons. Novelists, fan-fiction writers, game masters, and screenwriters borrow the idea for the same reason.",
        },
        {
          kind: "p",
          text: "A useful one tends to hold:",
        },
        {
          kind: "list",
          items: [
            "**Characters:** names, aliases, ages, appearances, relationships, and who is still alive.",
            "**Places:** geography, distances, what the harbor at Veld looks like at low tide.",
            "**Timeline:** the major events in story order, with dates or relative markers.",
            "**Rules:** how magic, technology, politics, or religion work, and what they cost.",
            "**Open questions:** the things you haven’t decided yet, written down so you know they’re open.",
          ],
        },
        {
          kind: "p",
          text: "The last item is the one people skip, and it may be the most useful. A list of undecided things is permission to keep writing without pretending you already know.",
        },
      ],
    },
    {
      id: "architects-and-gardeners",
      heading: "Are you an architect or a gardener?",
      blocks: [
        {
          kind: "p",
          text: "George R. R. Martin has described writers as architects or gardeners. The architect plans the house down to the plumbing before building anything. The gardener digs a hole, plants a seed, waters it, and finds out what comes up. Writing communities use a blunter pair of words for the same split: plotters and pantsers, the second group writing by the seat of their pants.",
        },
        {
          kind: "p",
          text: "An [architect](/glossary#architect) usually wants the bible first. Outlines, maps, family trees, a history of the war: the scaffolding is what makes the draft possible. A [gardener](/glossary#gardener) often finds that planning drains the story. Once they know what happens, the urge to write it fades. Their bible, if they keep one at all, grows behind the draft as a record of what the pages have already decided.",
        },
        {
          kind: "table",
          head: ["", "Architect", "Gardener"],
          rows: [
            ["Starts with", "An outline, a map, a timeline", "A voice, a scene, a question"],
            ["The bible is", "Built first, used as a plan", "Built after, used as a record"],
            ["Main risk", "Planning instead of writing", "Contradictions piling up by revision"],
            ["What helps", "A date to start the draft", "Settling details as pages decide them"],
          ],
          caption: "Two ways to work, and the bible each one tends to need.",
        },
        {
          kind: "p",
          text: "In practice, most writers are some of each. You might plan the plot and discover the characters, or know the ending and garden your way toward it. The useful question isn’t which label fits. It’s which details you need to decide *before* you write, and which you can safely decide *by* writing.",
        },
      ],
    },
    {
      id: "worldbuilding-instead-of-writing",
      heading: "Can worldbuilding become a way of not writing?",
      blocks: [
        {
          kind: "p",
          text: "Yes, and it’s one of the most pleasant ways to procrastinate ever devised. Designing a currency, a liturgy, and four centuries of succession disputes feels like progress. It *is* work. It just isn’t the book.",
        },
        {
          kind: "p",
          text: "Some signs the bible has started eating the draft:",
        },
        {
          kind: "list",
          items: [
            "You’ve written more words *about* the world than *in* it.",
            "You can’t start chapter 1 until you’ve settled the tax system of a country that first appears in chapter 30.",
            "You keep revising the bible instead of the manuscript.",
            "The bible holds things no scene will ever touch, and you know it.",
          ],
        },
        {
          kind: "p",
          text: "None of this means you should stop worldbuilding. Some people build worlds as a creative act in its own right, and that’s a perfectly good thing to do with an evening. It just means being honest about which project you’re working on today.",
        },
      ],
    },
    {
      id: "below-the-sand",
      heading: "How much of your world should the reader see?",
      blocks: [
        {
          kind: "p",
          text: "Picture your world as a pyramid half-buried in the desert. The reader sees the capstone catching the light: the handful of details a scene actually needs. Most of the stone stays below the sand.",
        },
        {
          kind: "p",
          text: "That buried mass isn’t wasted. It’s part of why the visible tip feels solid; readers sense depth in the confidence of a detail, even when they never see what’s under it. But the buried part doesn’t need to be finished. It needs to be *consistent where it shows*. A rough sketch underground is fine. A contradiction above ground is what readers notice.",
        },
        {
          kind: "quote",
          text: "The chain went up across the harbor at Veld the year the old queen died, and nobody had lowered it since.",
          cite: "Chapter 1, draft",
        },
        {
          kind: "p",
          text: "One sentence, and it implies a queen, a succession, a history of threat, and a city that has lived with a closed harbor for years. You don’t need to know all of that to write it. You need to remember it once it’s written, so that chapter 20 doesn’t lower the chain without a reason.",
        },
      ],
    },
    {
      id: "a-lean-story-bible",
      heading: "What should go in a lean story bible?",
      blocks: [
        {
          kind: "p",
          text: "A practical rule: write down what the pages have committed to, plus what the next few chapters need. Everything else can stay a sketch.",
        },
        {
          kind: "list",
          ordered: true,
          items: [
            "**Record details as the pages commit to them.** When a scene fixes an eye color, a death, or a distance, add it to the bible with the chapter number.",
            "**Mark how sure you are.** “Idea,” “rough,” and “decided” are enough. A bible that treats every note as law becomes a cage.",
            "**Keep open questions visible.** “Who killed the queen?” is a perfectly good entry.",
            "**Prune.** If an entry hasn’t mattered in fifty pages, it may not matter at all.",
            "**Read it before revision, not before every scene.** The bible checks the draft. It doesn’t need to stand guard over it.",
          ],
        },
      ],
    },
    {
      id: "starting-mid-draft",
      heading: "How do you start a story bible halfway through a draft?",
      blocks: [
        {
          kind: "p",
          text: "Plenty of writers only reach for a bible once the draft starts contradicting itself. That’s fine. A bible built from real pages is often better than one built from guesses. Just don’t try to build it in one sitting.",
        },
        {
          kind: "list",
          ordered: true,
          items: [
            "**Start with the people.** Read the draft once with only the cast in mind: names, ages, looks, relationships, and who dies. That alone catches the most visible slips.",
            "**Then the clock.** Make a one-line-per-scene list of when things happen, in one unit of time.",
            "**Then the places and rules,** but only the ones the pages have actually used.",
            "**Put a chapter number on everything.** When two entries disagree, the chapter numbers tell you where to look.",
          ],
        },
        {
          kind: "p",
          text: "When you find a contradiction while you’re harvesting, don’t fix it on the spot. Note both versions and keep reading. Deciding which one is true gets easier once you’ve seen the whole draft.",
        },
      ],
    },
    {
      id: "asten-optional-world",
      heading: "How does Asten handle a story bible?",
      blocks: [
        {
          kind: "p",
          text: "Asten treats the world as optional. You can write a whole book in the [desk](/features/desk) and never open the world view. If you do want one, [World](/features/world) offers three ways to look at the same material:",
        },
        {
          kind: "terms",
          items: [
            {
              term: "Wall",
              text: "A board for pinning loose scraps (a place, a question, a fragment) and moving them around until they want an order.",
            },
            {
              term: "Atlas",
              text: "A rough map where you can pin places. Crude shapes are enough; it’s a sketch, not a survey.",
            },
            {
              term: "Dossier",
              text: "The same entries gathered into a list by kind, for when you want to look something up.",
            },
          ],
        },
        {
          kind: "p",
          text: "Every world entry has a kind (place, culture, system, creature, item, question, or fragment) and a status that says how sure you are: **spark → sketched → settled**. A spark is a passing idea. A sketch is rough but usable. Settled means decided, and settling an entry can mint a confirmed [fact](/glossary#fact) in your [canon](/glossary#canon), which the compiler then weighs the draft against.",
        },
        {
          kind: "p",
          text: "That status line is the lean-bible rule, built in. Most of your world can stay a spark or a sketch, below the sand, and only the parts the story commits to need settling. Gardeners can settle details after the pages decide them; architects can sketch ahead and settle as they go. Either way, nothing in the world has to exist before you write.",
        },
        {
          kind: "callout",
          tone: "rubric",
          title: "A scene never waits on a wiki.",
          text: "Write first. Settle what the pages decide. Asten keeps the record; you keep the pen.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "Asten doesn’t pull facts out of your prose today; you settle them. The atlas has no travel-time checks, and there’s no graph view yet. Those are [planned](/roadmap), not shipped.",
        },
      ],
    },
    {
      id: "so-do-you-need-one",
      heading: "So, do you need a story bible?",
      blocks: [
        {
          kind: "p",
          text: "If you’re writing a short, linear novel with a small cast, a style sheet and a decent memory may be enough. If you’re writing a series, a shared world, fan fiction set against a large existing canon, or anything with rules that cost something, you probably want a bible. Make it a lean one, built mostly from what your pages have already decided.",
        },
        {
          kind: "p",
          text: "Either way, the bible serves the book. Architect or gardener, the measure is the same: are the pages getting written, and do they agree with each other? Everything else can stay below the sand.",
        },
      ],
    },
  ],
  faqs: [
    {
      q: "What is a story bible?",
      a: "A story bible is a reference document for a fictional world. It records characters, places, timelines, and rules so a long book or series stays consistent.",
    },
    {
      q: "What is the difference between a plotter and a pantser?",
      a: "A plotter plans the story before drafting; a pantser discovers it while drafting. The architect and gardener framing describes the same split, and most writers mix the two.",
    },
    {
      q: "How detailed should a story bible be?",
      a: "Detailed enough to keep the published pages consistent, and no more. Record what the draft has committed to, mark what is still undecided, and let the rest stay a sketch.",
    },
    {
      q: "Do I have to build a world before writing in Asten?",
      a: "No. Asten’s world view is optional, and a scene never waits on it. World entries move from spark to sketched to settled as you decide them.",
    },
  ],
  related: [
    "/features/world",
    "/for/worldbuilders",
    "/for/fan-fiction",
    "/journal/continuity-in-a-long-novel",
    "/glossary",
  ],
};
