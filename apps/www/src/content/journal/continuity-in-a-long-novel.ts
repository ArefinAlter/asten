import type { Article } from "../types";
import { journalMeta } from "./meta";

export const article: Article = {
  ...journalMeta["continuity-in-a-long-novel"]!,
  answer:
    "Keep continuity in a long novel by deciding which details are settled, recording them in one place, and checking each new chapter against that record. A style sheet, a story bible, and a scene list catch most slips by hand. A canon checker can mark contradictions for you once those facts are settled.",
  takeaways: [
    "Most continuity slips fall into six families: attributes, lifecycles, timeline, names, objects, and places.",
    "A style sheet, a scene list, and targeted search passes catch most of them by hand.",
    "Check at the seams: after a draft, after moving scenes, and before the book goes out.",
    "The most useful step is settling a detail once, in one place, so later pages have something to be checked against.",
    "Asten marks contradictions against facts you’ve confirmed; deciding which side is right stays with you.",
  ],
  sections: [
    {
      id: "why-continuity-slips",
      heading: "Why does continuity slip in a long novel?",
      blocks: [
        {
          kind: "p",
          text: "Nobody loses track of a heroine’s hair color on purpose. It happens because a novel gets written over months or years, in hundreds of sittings, by a person whose memory is busy with harder things: the shape of the second act, the line of dialogue that won’t sit right, the subplot that might have to go. Chapter 2 was written last spring. Chapter 19 was written at midnight last Tuesday. The two have never met.",
        },
        {
          kind: "p",
          text: "Revision makes it worse, not better. You cut the scene where Oren loses his ship and forget that chapter 14 still mentions the loss. You rename a city and miss three of the forty mentions.",
        },
        {
          kind: "p",
          text: "Readers, meanwhile, are very good at noticing. They read your book in a week, so details you set down a year apart sit side by side in their heads. The goal of any continuity process is simple to state: give your memory the same advantage the reader’s has.",
        },
      ],
    },
    {
      id: "kinds-of-continuity-errors",
      heading: "What kinds of continuity errors are there?",
      blocks: [
        {
          kind: "p",
          text: "Most slips fall into a handful of families. Knowing which one you’re hunting makes the hunt faster.",
        },
        {
          kind: "table",
          head: ["Kind", "Example", "Where it hides"],
          rows: [
            [
              "Attributes",
              "Mira’s auburn hair turns black in chapter 19",
              "Descriptions, ages, heights, eye colors, accents",
            ],
            [
              "Lifecycle",
              "Captain Oren dies in chapter 9 and gives an order in chapter 14",
              "Deaths, births, injuries, marriages, departures",
            ],
            [
              "Timeline",
              "A three-day ride takes one day on the way back",
              "Travel, pregnancies, seasons, “the next morning”",
            ],
            [
              "Names and aliases",
              "“The Captain,” “Oren,” and “Orren” on the same page",
              "Nicknames, titles, spelling drift, renamed characters",
            ],
            [
              "Objects",
              "The sword lost in the river is back on her hip",
              "Weapons, letters, keys, rings, money",
            ],
            [
              "Places",
              "The harbor at Veld faces east, then west",
              "Geography, room layouts, which door leads where",
            ],
          ],
          caption: "Six families of continuity error, with an example of each.",
        },
        {
          kind: "p",
          text: "Some of these are pure memory problems (attributes, names). Some are arithmetic (timeline). Lifecycle errors are the most embarrassing, because they’re the easiest for a reader to spot. Dead is a yes-or-no question.",
        },
      ],
    },
    {
      id: "checking-by-hand",
      heading: "How do you check continuity by hand?",
      blocks: [
        {
          kind: "p",
          text: "Writers and editors kept long books straight for a long time before software came along. The old tools still work.",
        },
        {
          kind: "h3",
          text: "Keep a style sheet",
        },
        {
          kind: "p",
          text: "Copyeditors keep a style sheet for every book: how names are spelled, which words are capitalized, how numbers are written, and a running list of character details. Start your own on day one. Keep it as a plain list, alphabetical by name, with a chapter number beside every entry. The chapter number is what turns a memory into evidence.",
        },
        {
          kind: "h3",
          text: "Keep a story bible, if it helps",
        },
        {
          kind: "p",
          text: "A [story bible](/glossary#story-bible) is a bigger reference: characters, places, history, and the rules of the world. It earns its keep for series, shared worlds, and anything with a magic system. Keep it lean; it exists to serve the draft. (There’s more on that in [Do you need a story bible?](/journal/do-you-need-a-story-bible))",
        },
        {
          kind: "h3",
          text: "Make a scene list",
        },
        {
          kind: "p",
          text: "One line per scene: chapter, point-of-view character, where it happens, when it happens, and who’s present. Timeline errors show up here, because you can see the days side by side. Add a column for who is alive.",
        },
        {
          kind: "h3",
          text: "Run search passes",
        },
        {
          kind: "p",
          text: "Before a draft goes out, search for every name, alias, and plausible misspelling. Search for colors near character names. Search for “the next morning” and “that night” and check each one against your scene list. It’s tedious, and it works.",
        },
        {
          kind: "h3",
          text: "Ask beta readers specific questions",
        },
        {
          kind: "p",
          text: "Beta readers catch what you can no longer see, but “Any continuity problems?” mostly gets a shrug. Ask instead: “Did anything about timing or travel feel off?” “Did anyone seem to know something they shouldn’t?” “Did you ever lose track of who was in the room?”",
        },
      ],
    },
    {
      id: "when-to-check",
      heading: "When should you check for continuity errors?",
      blocks: [
        {
          kind: "p",
          text: "Not while you’re drafting, if checking slows you down. A first draft’s job is to exist, and stopping every paragraph to confirm an eye color is a reliable way to lose a scene. Plenty of writers keep a running “verify later” note and push on.",
        },
        {
          kind: "p",
          text: "The best moments to check are at the seams of the work:",
        },
        {
          kind: "list",
          items: [
            "**After a draft.** Build or update the style sheet from what the pages actually say, not what you remember them saying.",
            "**After cutting or moving scenes.** Recheck the scene list and every reference to what you cut. Moving scenes is the riskiest edit of all; a reordered book can quietly put a funeral before a death.",
            "**Before it goes out.** Run your search passes before the manuscript reaches beta readers, an editor, or an agent.",
          ],
        },
      ],
    },
    {
      id: "settling-facts-as-canon",
      heading: "What does it mean to settle a fact as canon?",
      blocks: [
        {
          kind: "p",
          text: "All of those methods share one move: deciding that a detail is fixed, and writing it down somewhere other than the manuscript. That’s what [canon](/glossary#canon) means here. Not “everything in the book,” but the details you’ve decided are true.",
        },
        {
          kind: "p",
          text: "The difference matters. A draft is full of details nobody has decided yet. Mira’s hair might be auburn only because you needed an adjective at one in the morning. Settling it is a separate act: “Yes. Auburn. That’s her.” Once a detail is settled, a later page that disagrees is a question worth asking. Before it’s settled, it’s just a draft being a draft.",
        },
        {
          kind: "callout",
          tone: "settled",
          title: "Settled so far",
          text: "Mira · hair · auburn (chapter 2). Captain Oren · lifecycle · dead (chapter 9). The harbor at Veld · faces · east (chapter 3).",
        },
        {
          kind: "p",
          text: "A tool can only help once you’ve made that move. In Asten’s [canon](/features/canon), a [fact](/glossary#fact) is an entity, a property, and a value (*Mira · hair · auburn*) with a status. Only confirmed facts produce hard contradictions, so half-formed ideas don’t start arguing with you. A fact can also carry a valid-from and a valid-to, which is how you record a change, like a dye job or a promotion, rather than a mistake. And when you change your mind about the record, you mark the old fact contested or deprecated, and it stops arguing.",
        },
      ],
    },
    {
      id: "worked-example",
      heading: "What does a continuity check look like in practice?",
      blocks: [
        {
          kind: "p",
          text: "Take the two classic slips from our running example.",
        },
        {
          kind: "h3",
          text: "Mira’s hair",
        },
        {
          kind: "p",
          text: "In chapter 2 you settled Mira’s hair as auburn. Seventeen chapters and several months later, you write:",
        },
        {
          kind: "quote",
          text: "She pushed her black hair out of her eyes and looked down at the harbor.",
          cite: "Chapter 19, draft",
        },
        {
          kind: "p",
          text: "Asten’s [compiler](/features/compiler) weighs the page against the record and marks it in red:",
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
          text: "The mark doesn’t fix anything. It names the rule, the sentence, and the settled fact, and leaves you the choice. *Keep my sentence* if chapter 19 is right and you’ll revisit chapter 2. *Retire “auburn”* if black is the new truth. *Add an event* if Mira dyed her hair and the story should show it. Or [suppress](/glossary#suppression) the mark with a reason; the suppression is saved with the project, so it won’t come back on the next pass.",
        },
        {
          kind: "h3",
          text: "Captain Oren’s death",
        },
        {
          kind: "p",
          text: "Oren dies in chapter 9, and you settled it. In chapter 14, a scene beat has him on the quay, giving orders.",
        },
        {
          kind: "diagnostic",
          code: "CANON-E101",
          title: "Captain Oren acts after a settled death.",
          lines: [
            "① Captain Oren dies (day 9)",
            "② Chapter 14 mentions Captain Oren in narration: “Captain Oren”",
            "Mark the scene as a memory · Set the scene’s time · Move the death later · Silence this mark · Keep my sentence",
          ],
        },
        {
          kind: "p",
          text: "Maybe chapter 14 is a memory and should read like one. Maybe someone else gives the order. Maybe Oren isn’t as dead as everyone thinks, and that’s the twist, in which case you silence the rule for him and move on. The attribute check is always on. The lifecycle check lives in the Lifecycle [method pack](/glossary#method-pack), which is on by default and can be turned off.",
        },
      ],
    },
    {
      id: "what-a-checker-cannot-catch",
      heading: "What can’t a continuity checker catch?",
      blocks: [
        {
          kind: "p",
          text: "Be wary of any tool that promises to catch everything. Asten doesn’t. It reads only plain attributes from your prose, such as hair color, age, or whether someone is alive, and only when a name is in the same sentence; a pronoun on its own is not weighed, and what it reads is never stored as a fact. Travel checks work only along routes you declare with a travel time. Name drift is still partly a job for your search passes, though Asten flags names that aren’t in the cast and names that are easy to confuse, and giving an entity its aliases means “the Captain” and Oren are recorded as the same person.",
        },
        {
          kind: "p",
          text: "And no checker can tell you whether a contradiction is a mistake, a lie a character tells, or a clue you planted. That judgment is the writing. A good process (a style sheet, a scene list, a handful of settled facts, and something to do the comparing) just makes sure you get to exercise it before your readers do.",
        },
        {
          kind: "callout",
          tone: "rubric",
          title: "What a mark does",
          text: "Marks point at a sentence and the settled fact it disagrees with. They never rewrite it. No AI model runs in the checks; they are deterministic rules over facts you confirm.",
        },
      ],
    },
  ],
  faqs: [
    {
      q: "What is a continuity error in a novel?",
      a: "A continuity error is a detail that contradicts something the story already established, such as a character’s hair color changing, a dead character reappearing, or a journey taking a different length of time on the way back.",
    },
    {
      q: "What is a style sheet for a novel?",
      a: "A style sheet is a running list of a book’s spellings, capitalization choices, and character details, each with the chapter where it was set. Copyeditors keep one for every book, and writers can keep their own.",
    },
    {
      q: "Can software find continuity errors automatically?",
      a: "Software can compare new pages against details you have settled. Asten marks contradictions against confirmed facts. It also reads plain attributes from the page with fixed rules and weighs them against your facts, but it never settles anything; you decide what is canon.",
    },
    {
      q: "How does Asten handle a character who changes, like dyed hair?",
      a: "A fact can carry a valid-from and a valid-to, so a change is recorded as a change. When a mark appears, you can keep your sentence, update canon, add an event, or suppress the mark with a reason.",
    },
  ],
  related: [
    "/features/canon",
    "/features/compiler",
    "/features/cast",
    "/journal/story-time-and-reading-order",
    "/for/novelists",
  ],
};
