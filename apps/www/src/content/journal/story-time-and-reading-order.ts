import type { Article } from "../types";
import { journalMeta } from "./meta";

export const article: Article = {
  ...journalMeta["story-time-and-reading-order"]!,
  answer:
    "Story time is when events happen in the story’s world; reading order is when the reader meets them on the page. Flashbacks, frame stories, and openings in the middle of the action separate the two on purpose. A contradiction exists only when two things you’ve stated about time can’t both be true.",
  takeaways: [
    "Story time is when it happens; reading order is when the reader finds out.",
    "Flashbacks, frames, and parallel timelines are deliberate gaps between the two clocks, not errors.",
    "Track both clocks side by side in one scene list, using one unit of time.",
    "A real contradiction is two statements about time that can’t both be true.",
    "Asten’s CANON-E201 compares the two clocks, and it stays silent until both exist.",
  ],
  sections: [
    {
      id: "two-clocks",
      heading: "What is the difference between story time and reading order?",
      blocks: [
        {
          kind: "p",
          text: "Every story runs on two clocks. The first is [story time](/glossary#story-time): the order in which things happen in the world of the story. Oren is born, Oren goes to sea, Oren takes command, the harbor burns, Oren dies. The second is [reading order](/glossary#reading-order): the order in which the reader meets those events on the page. You might open with the fire, cut back to his first voyage, and let the reader learn of his death from a letter.",
        },
        {
          kind: "p",
          text: "Same events, two sequences. Neither one is wrong. The gap between them is one of the main tools you have for suspense, mystery, and emotional timing: what the reader knows, and when, is as much a part of the story as what happens.",
        },
        {
          kind: "p",
          text: "The idea has formal names. Early twentieth-century Russian critics, usually grouped as the Russian Formalists, distinguished the *fabula*, the raw events in chronological order, from the *syuzhet*, the way a telling arranges them. Critics have used versions of that distinction ever since. You don’t need the vocabulary. You need the habit of remembering that there are two clocks.",
        },
        {
          kind: "terms",
          items: [
            {
              term: "Story time (fabula)",
              text: "When events happen in the story’s world, in chronological order.",
            },
            {
              term: "Reading order (syuzhet)",
              text: "When the reader meets those events, in the order of the pages.",
            },
            {
              term: "Flashback",
              text: "A scene that comes later in reading order than its place in story time.",
            },
            {
              term: "Flash-forward",
              text: "A scene that shows a later moment of story time before the events leading up to it.",
            },
          ],
        },
      ],
    },
    {
      id: "splitting-the-clocks",
      heading: "How do flashbacks, frames, and parallel timelines work?",
      blocks: [
        {
          kind: "h3",
          text: "Flashbacks",
        },
        {
          kind: "p",
          text: "A flashback steps backward in story time while reading order keeps moving forward. The craft questions are about the doorways: a clear cue going in (a shift of tense, a dated heading, a smell or sound that pulls the character back), a reason the past matters right now, and a clean way out. The continuity question is narrower. Whatever happens in the flashback has to fit what you’ve already established about that earlier time.",
        },
        {
          kind: "h3",
          text: "Frame stories",
        },
        {
          kind: "p",
          text: "A frame story wraps one telling inside another: someone in the present tells, writes, or remembers the main story. *Frankenstein* is framed by a sea captain’s letters home; *One Thousand and One Nights* by a storyteller buying herself one more night. Frames give you two story-time lines at once, the teller’s and the tale’s, and each needs its own consistent clock.",
        },
        {
          kind: "h3",
          text: "Parallel timelines",
        },
        {
          kind: "p",
          text: "Some books alternate between two eras, or between two characters whose stories happen years apart and slowly converge. Each strand has its own internal order. The places where the strands touch (an heirloom handed down, a house that burns in one strand and still stands in the other) are where contradictions breed.",
        },
        {
          kind: "h3",
          text: "In medias res",
        },
        {
          kind: "p",
          text: "Starting “in the middle of things,” to borrow the old Latin phrase, means opening at a dramatic moment and filling in what came before. It’s the most common split between the clocks and the easiest to lose track of, because the backstory arrives in fragments scattered across many chapters.",
        },
        {
          kind: "quote",
          text: "By the time the harbor at Veld caught fire, Captain Oren had already made the mistake that would kill him, and Mira was the only one who knew it.",
          cite: "Chapter 1, opening line (draft)",
        },
        {
          kind: "p",
          text: "One sentence, two clocks. The fire is now. The mistake is earlier, and the reader will learn about it later. The death is still to come, and the reader will spend eight chapters waiting for it. Every one of those has to stay true for the rest of the book.",
        },
      ],
    },
    {
      id: "tracking-two-clocks",
      heading: "How do you track two clocks without losing your mind?",
      blocks: [
        {
          kind: "p",
          text: "The simplest tool is a scene list with two columns you keep strictly separate: where each scene sits in the book, and when it happens in the world.",
        },
        {
          kind: "table",
          head: ["Reading order", "Scene", "Story time", "Note"],
          rows: [
            ["1", "The harbor burns", "Year 12, late autumn", "Opening, in the middle of things"],
            ["2", "Oren’s first voyage", "Year 1, spring", "Flashback"],
            ["3", "Mira reaches Veld", "Year 12, late autumn", "The morning after the fire"],
            ["4", "Oren’s death", "Year 12, early winter", "Chapter 9; the death the opening hinted at"],
          ],
          caption: "A scene list that keeps both clocks visible.",
        },
        {
          kind: "p",
          text: "A few habits make a list like this earn its keep:",
        },
        {
          kind: "list",
          items: [
            "**Pick one unit and stick to it.** Days from the start of the story, a calendar date, or a season and a year. Mixing units is how “three weeks later” quietly becomes two.",
            "**Give every scene a story time, even the linear ones.** Most contradictions hide in the scenes you assumed were obvious.",
            "**Label deliberate jumps.** A flashback marked as one is a choice. An unmarked backward jump is a question.",
            "**Note your anchors.** Births, deaths, wars, holidays, phases of the moon. Everything else gets measured from them.",
          ],
        },
        {
          kind: "h3",
          text: "Two strands, two lists",
        },
        {
          kind: "p",
          text: "For a frame story or parallel timelines, keep one list per strand, each in its own story-time order, and then a merged list in reading order that shows how the strands alternate. The strand lists catch slips inside a timeline. The merged list catches the ones between them, like an heirloom that turns up in the past strand after it was lost in the present one.",
        },
        {
          kind: "h3",
          text: "On the screen",
        },
        {
          kind: "p",
          text: "Screenwriters face the same problem with fewer words to solve it. The audience learns story time from a title card, a costume, a haircut, a season outside the window. Your scene list is where you make sure those cues agree, so the flashback that’s supposed to be ten years earlier doesn’t put a character in a coat they only buy in act two.",
        },
      ],
    },
    {
      id: "when-a-contradiction-is-real",
      heading: "When is a timeline contradiction real?",
      blocks: [
        {
          kind: "p",
          text: "A flashback isn’t a contradiction. Neither is a frame, a parallel strand, or a cold open. Those are gaps between the clocks that you put there on purpose. A contradiction is when two things you’ve stated about time can’t both be true. For example:",
        },
        {
          kind: "list",
          items: [
            "Chapter 11 opens “the morning after the funeral,” but your notes put chapter 11 a day *before* the funeral.",
            "Mira reaches Veld in late autumn in her own strand and in midwinter in Oren’s, and both are meant to be the same arrival.",
            "In a flashback, a character remembers talking with someone who hadn’t been born yet.",
            "The frame narrator, telling the tale in Year 12, describes the rebuilt harbor, which the main story doesn’t rebuild until Year 13.",
          ],
        },
        {
          kind: "p",
          text: "The test is simple. If you can’t explain the gap as a choice, it’s probably a slip. If you *can* explain it (the narrator is lying, the memory is wrong on purpose), then it’s a choice, and you should record it as one so that future you doesn’t “fix” it during revision.",
        },
      ],
    },
    {
      id: "how-asten-tracks-time",
      heading: "How does Asten keep track of the two clocks?",
      blocks: [
        {
          kind: "p",
          text: "Asten’s [Time](/features/time) view keeps both clocks as separate things: story time, for when something happens, and reading order, for when the reader meets it. You don’t have to choose one view of your book. You state each, and the compiler compares them.",
        },
        {
          kind: "p",
          text: "The rule that does the comparing is **CANON-E201**, part of the Clocks [method pack](/glossary#method-pack), which is on by default. It stays silent until both clocks exist. If you never state a story time, you never hear from it. When the clocks you’ve stated disagree, it marks the spot:",
        },
        {
          kind: "diagnostic",
          code: "CANON-E201",
          title: "Reading order and story time disagree.",
          lines: [
            "Chapter 10 · Oren’s funeral · story time: day 15",
            "Chapter 11 · The morning after · story time: day 14",
            "Reorder the binder · Retcon a date · Silence this rule",
          ],
        },
        {
          kind: "p",
          text: "As with every Asten [diagnostic](/glossary#diagnostic), the mark points and never rewrites. Maybe the scenes are in the wrong order. Maybe a date is off by one. Or maybe the jump is deliberate, and that’s fine too: flashbacks are allowed. You account for a deliberate jump by [suppressing](/glossary#suppression) the mark for that scene, with a reason, and the suppression is saved with the project so the same question doesn’t come back on every pass.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "Asten doesn’t read time out of your prose today; you state each scene’s story time yourself. Fact proposals drawn from your pages and custom calendars are both [planned](/roadmap). No AI model runs in Asten today.",
        },
      ],
    },
    {
      id: "nonlinear-checklist",
      heading: "What should you check before a nonlinear draft goes out?",
      blocks: [
        {
          kind: "list",
          ordered: true,
          items: [
            "Every scene has a story time, in one consistent unit.",
            "Every backward jump is either labeled as a flashback or fixed.",
            "Every relative phrase (“the next morning,” “a week later”) agrees with the scene list.",
            "Every character in a flashback is alive, and old enough, at that point in story time.",
            "Every place where two timelines touch has been read side by side.",
            "Every deliberate inconsistency, like a lie or a false memory, is recorded as a choice.",
          ],
        },
        {
          kind: "p",
          text: "None of this flattens a structure. If anything, it frees you to be more ambitious with one, because you know the scaffolding will hold. Write the braided timeline. Open in the fire. Just keep both clocks where you can see them.",
        },
      ],
    },
  ],
  faqs: [
    {
      q: "What is the difference between fabula and syuzhet?",
      a: "Fabula is a story’s events in chronological order; syuzhet is the order and manner in which the telling presents them. The terms come from the Russian Formalist critics of the early twentieth century.",
    },
    {
      q: "Is a flashback a continuity error?",
      a: "No. A flashback is a deliberate gap between story time and reading order. It becomes a problem only when what happens in it contradicts something already established about that earlier time.",
    },
    {
      q: "How do I keep track of a nonlinear timeline?",
      a: "Keep a scene list with two separate columns, one for where each scene sits in the book and one for when it happens in the story, and use one consistent unit of time throughout.",
    },
    {
      q: "When does Asten’s CANON-E201 rule mark a scene?",
      a: "CANON-E201 marks a scene when the reading order and story time you have stated contradict each other. It stays silent until both clocks exist, and any mark can be suppressed with a reason.",
    },
  ],
  related: [
    "/features/time",
    "/features/packs",
    "/journal/continuity-in-a-long-novel",
    "/for/screenwriters",
    "/glossary",
  ],
};
