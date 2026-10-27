import type { Article } from "../types";
import { journalMeta } from "./meta";

export const article: Article = {
  ...journalMeta["local-first-writing"]!,
  answer:
    "Your manuscript should live on your own computer because a local file keeps working when the internet, a company, or a policy doesn’t. Local-first software keeps the real copy on your device, works offline, and lets you export to open formats. Add a 3-2-1 backup and your book outlives any single tool.",
  takeaways: [
    "Local-first means your device holds the real copy; any cloud service is optional, not the owner.",
    "Cloud-only tools add risks you don’t control: lockouts, outages, shutdowns, and changing terms.",
    "Follow the 3-2-1 rule: three copies, on two kinds of storage, with one somewhere else.",
    "Export regularly to open formats such as Markdown, HTML, and .docx.",
    "Asten keeps your projects in a SQLite file on your computer, with no account, no sync, and no telemetry.",
  ],
  sections: [
    {
      id: "where-your-book-lives",
      heading: "Where does your manuscript actually live?",
      blocks: [
        {
          kind: "p",
          text: "Ask a writer where their novel is and they’ll usually point at a laptop. Often the honest answer is different: on a server in a data center they’ve never seen, owned by a company whose terms they skimmed once, reachable as long as the login works and the internet is up. The laptop is a window.",
        },
        {
          kind: "p",
          text: "For a grocery list, that’s fine. For a book you’ve worked on for three years, a fan-fiction series readers are waiting on, or a screenplay in its fortieth revision, it’s worth asking whether a window is enough.",
        },
        {
          kind: "p",
          text: "Some tools blur the line. A desktop app might keep a local cache while treating the server as the truth, or offer a download in a format nothing else can open. There’s a simple test: turn off the internet, sign out, and see what you can still open and edit. Whatever’s left is what you actually own.",
        },
      ],
    },
    {
      id: "what-is-local-first",
      heading: "What is local-first software?",
      blocks: [
        {
          kind: "p",
          text: "[Local-first](/glossary#local-first) is a name for software that treats the copy on your own device as the real one. The idea was set out in a widely read 2019 essay from the research lab Ink & Switch, which argued that you can keep much of the convenience of cloud apps without giving up ownership of your work. Paraphrased, their wish list looks something like this:",
        },
        {
          kind: "list",
          items: [
            "**It’s fast,** because your work isn’t waiting on a round trip to a server.",
            "**It works offline,** on a plane, in a cabin, or during an outage.",
            "**It lasts,** so your files stay readable long after a product or a company is gone.",
            "**It’s private** by default.",
            "**You’re in control:** you can copy, back up, and delete your own data.",
            "**It can still sync and collaborate,** without making a server the owner of your work.",
          ],
        },
        {
          kind: "p",
          text: "Notice the last item. Local-first isn’t anti-internet. It’s about who holds the master copy. A local-first app can still sync between devices; it just doesn’t *need* a server’s permission to let you open your own book.",
        },
      ],
    },
    {
      id: "risks-of-cloud-only",
      heading: "What can go wrong with cloud-only writing tools?",
      blocks: [
        {
          kind: "p",
          text: "Cloud tools are convenient, and plenty of writers use them happily. The risks are real, though, and they share one feature: you don’t control them.",
        },
        {
          kind: "list",
          items: [
            "**Lockouts.** A forgotten password, a lost phone holding your two-factor codes, a flagged account, or a lapsed payment can stand between you and your manuscript.",
            "**Outages.** When the service is down, so is your writing session.",
            "**Shutdowns and redesigns.** Products get discontinued, sold, or rebuilt. Features you rely on can disappear, and export options aren’t guaranteed to stay.",
            "**Policy changes.** Terms of service change. Many writers now read them looking for one thing: whether their words can be used to train AI models.",
            "**Formats you can’t open anywhere else.** If the only way to read your work is through one company’s app, your book is only as durable as that company.",
          ],
        },
        {
          kind: "p",
          text: "None of this is a scandal. It’s simply what it means to keep your work in someone else’s house.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "If you use a cloud tool, find its export button today, not on the day you need it. Then press it.",
        },
      ],
    },
    {
      id: "privacy-and-training",
      heading: "Why does it matter whether a company can see your pages?",
      blocks: [
        {
          kind: "p",
          text: "For many writers, the sharpest worry about the cloud is a newer one: that unpublished pages could end up as training data for an AI model. Policies differ from service to service and change over time, and few people can keep track of every clause. Reading the terms is still worth doing.",
        },
        {
          kind: "p",
          text: "Local storage answers the question in a simpler way. A company can’t train on a manuscript it never receives. When the only copy lives on your machine and the software sends nothing home, privacy stops being a matter of trust and fine print and becomes a matter of where the file is. (The one exception is the assistant you may switch on: it sends a request to the provider you chose, only after you consent, and the [AI policy](/ai-policy) lists exactly what goes.)",
        },
        {
          kind: "p",
          text: "That doesn’t make every desktop app private. Software on your computer can still send data out. The questions to ask any tool are the same: Does it need an account? Does it sync, and to where? Does it report on how you use it? Does it still work with the network unplugged?",
        },
      ],
    },
    {
      id: "backing-up-a-manuscript",
      heading: "How should you back up a manuscript?",
      blocks: [
        {
          kind: "p",
          text: "Local-first moves the responsibility to you, which is both the point and the catch. A file on one laptop is one spilled coffee away from gone. The standard answer is the 3-2-1 rule, a rule of thumb that photographers and IT people have repeated for years:",
        },
        {
          kind: "list",
          items: [
            "**Three copies** of your work: the one you edit, plus two backups.",
            "**Two kinds of storage,** for example your laptop’s drive and an external drive.",
            "**One copy somewhere else:** a drive at a friend’s house, a desk drawer at work, or an encrypted online backup.",
          ],
        },
        {
          kind: "p",
          text: "That last item deserves a note. A cloud *backup* is different from a cloud-*only* tool. A backup is a copy you control, in a format you can open, that you don’t need in order to keep writing.",
        },
        {
          kind: "p",
          text: "Two more habits make the rule work in real life:",
        },
        {
          kind: "list",
          ordered: true,
          items: [
            "**Automate it.** Most computers come with a backup tool that runs on a schedule. A backup that depends on remembering is a backup that stops.",
            "**Test a restore.** Every so often, open a backup on a different machine. A backup you have never opened may not work.",
          ],
        },
      ],
    },
    {
      id: "formats-that-last",
      heading: "Which file formats will still open in twenty years?",
      blocks: [
        {
          kind: "p",
          text: "Ownership isn’t only about where a file lives. It’s also about whether anything can still read it. Plain, open formats age best.",
        },
        {
          kind: "table",
          head: ["Format", "Good for", "Why it lasts"],
          rows: [
            ["Plain text (.txt)", "Raw words and archives", "Almost any device can open it"],
            ["Markdown (.md)", "Drafts with light formatting", "Plain text with simple, readable marks"],
            ["HTML", "Formatted reading copies; print to PDF", "Every web browser reads it"],
            ["Fountain", "Screenplays", "A plain-text screenplay format with a public spec"],
            [".docx", "Editors, agents, submissions", "The format much of publishing expects"],
          ],
          caption: "Open formats worth exporting your manuscript to.",
        },
        {
          kind: "p",
          text: "A good habit: at every milestone (the end of a draft, the night before a big revision), export a plain copy and drop it into your backups. If every app you use vanished tomorrow, you would still have the book.",
        },
      ],
    },
    {
      id: "how-asten-keeps-it-local",
      heading: "How does Asten keep your manuscript on your computer?",
      blocks: [
        {
          kind: "p",
          text: "Asten is built [local-first](/features/local-first). Here is exactly what that means today:",
        },
        {
          kind: "list",
          items: [
            "**Your project is a file on your disk.** Asten stores it in SQLite, a widely used database format, in a file called `asten.db` in your operating system’s application-data folder when you use the desktop app.",
            "**No account.** There’s nothing to sign in to.",
            "**No sync and no telemetry.** Asten doesn’t send reports about how you use it anywhere, and it sends your work nowhere unless you switch on the assistant and ask it a question. Verne, the company that makes Asten, never receives your manuscript.",
            "**Open exports.** [Compile](/features/word-and-google-docs) to Markdown, plain text, Fountain, HTML, .docx, EPUB 3, or Final Draft (.fdx). For a PDF, print the HTML page. Import reads the same formats except EPUB.",
            "**No training.** No manuscript is used to train any model, and the checks run no AI model. The [AI policy](/ai-policy) and [privacy notice](/privacy) say so in full.",
          ],
        },
        {
          kind: "p",
          text: "Inside that file, named snapshots and an append-only history in the [desk](/features/desk) let you step back through your own changes. That protects you from yourself (the scene you cut too eagerly) but not from a dead drive. Snapshots live in the same file as your book, so `asten.db` belongs in your 3-2-1 backups like everything else, along with a compiled Markdown or .docx copy you can open anywhere.",
        },
        {
          kind: "callout",
          tone: "rubric",
          title: "The house is yours.",
          text: "Your words stay on your computer, in a file you can copy.",
        },
        {
          kind: "callout",
          tone: "note",
          text: "Local-first has trade-offs. Asten has no sync, no collaboration, and no app in the phone app stores today; collaboration is [planned](/roadmap) without giving up offline work. Asten is in early development; release 0.0.1b is a public pre-alpha for Windows and Linux. See [Download](/download).",
        },
      ],
    },
    {
      id: "a-simple-plan",
      heading: "What’s a simple plan for keeping your book safe?",
      blocks: [
        {
          kind: "list",
          ordered: true,
          items: [
            "Know where the real copy of your manuscript lives.",
            "Make sure you can open it without an internet connection or an account.",
            "Export to an open format at every milestone.",
            "Keep three copies on two kinds of storage, with one somewhere else.",
            "Read the data and AI-training terms of any tool that holds your words.",
          ],
        },
        {
          kind: "p",
          text: "The manuscript is the one thing in this process you can’t replace, so keep it somewhere you can see it, in a form you can open.",
        },
      ],
    },
  ],
  faqs: [
    {
      q: "What does local-first mean?",
      a: "Local-first software keeps the main copy of your work on your own device, works without an internet connection, and treats any cloud service as optional rather than as the owner of your data.",
    },
    {
      q: "What is the 3-2-1 backup rule?",
      a: "Keep three copies of your work on two different kinds of storage, with at least one copy in a different physical place.",
    },
    {
      q: "Where does Asten store my manuscript?",
      a: "In a SQLite database file, asten.db, in your operating system’s application-data folder when you use the desktop app. There is no account, no sync, and no telemetry.",
    },
    {
      q: "Which formats can Asten export?",
      a: "Asten compiles to Markdown, plain text, Fountain, HTML, .docx, EPUB 3, and Final Draft (.fdx). For a PDF, you print the HTML page.",
    },
  ],
  related: [
    "/features/local-first",
    "/privacy",
    "/ai-policy",
    "/features/compiler",
    "/journal/ai-editor-not-writer",
  ],
};
