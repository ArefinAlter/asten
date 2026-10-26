import { describe, expect, it } from "vitest";
import { TOOLS } from "../agent/tools";
import {
  applyEvent,
  emptyTurn,
  forStorage,
  spendLine,
  type AiEvent,
  type Totals,
} from "./bridge";

/** The Rust table (`crates/asten-assistant/src/policy.rs`) lists these and no others. */
const RUST_TOOLS = [
  "marks.list",
  "marks.explain",
  "checks.run",
  "canon.search",
  "graph.search",
  "scene.summary",
  "timeline.list",
  "sources.summarise",
  "facts.propose",
  "repairs.propose",
  "manuscript.write",
  "manuscript.rewrite",
  "manuscript.continue",
  "facts.settle",
  "settings.change",
];

describe("the assistant's tool table", () => {
  it("is the same closed table on both sides of the bridge", () => {
    expect(TOOLS.map((t) => t.name)).toEqual(RUST_TOOLS);
    const closed = TOOLS.filter((t) => t.needs === "never").map((t) => t.name);
    expect(closed).toEqual([
      "manuscript.write",
      "manuscript.rewrite",
      "manuscript.continue",
      "facts.settle",
      "settings.change",
    ]);
  });
});

const totals: Totals = {
  day_tokens: 1200,
  day_input: 1000,
  day_output: 200,
  day_usd: 0.0123,
  day_calls: 2,
  thread_tokens: 600,
  thread_usd: 0.004,
  day_has_unpriced: false,
};

function fold(events: AiEvent[]) {
  let turn = emptyTurn();
  let text = "";
  for (const event of events) ({ turn, text } = applyEvent(turn, text, event));
  return { turn, text };
}

describe("a model run, as the panel folds it", () => {
  it("streams text, lists tool calls with their results and cards, and sums usage", () => {
    const { turn, text } = fold([
      {
        type: "started",
        run_id: "r",
        provider: "anthropic",
        name: "Anthropic",
        model: "claude-haiku-5-5",
        host: "api.anthropic.com",
        local: false,
      },
      {
        type: "request",
        n: 1,
        provider: "anthropic",
        host: "api.anthropic.com",
        bytes: 4100,
        sha256: "ab".repeat(32),
        tokens_estimate: 1000,
        body: "{}",
      },
      {
        type: "tool_call",
        call_id: "t1",
        tool: "marks.list",
        input: {},
        allowed: true,
        reason: null,
      },
      {
        type: "tool_result",
        call_id: "t1",
        tool: "marks.list",
        summary: "3 marks",
        ms: 12,
        is_error: false,
      },
      { type: "card", card: { type: "marks", ids: ["a"] } },
      {
        type: "usage",
        input: 300,
        output: 40,
        cost_usd: 0.0005,
        estimated: false,
        totals,
      },
      { type: "text", delta: "Three " },
      { type: "text", delta: "marks." },
      {
        type: "usage",
        input: 500,
        output: 60,
        cost_usd: 0.0007,
        estimated: false,
        totals,
      },
      { type: "done", run_id: "r", requests: 2, tool_calls: 1 },
    ]);
    expect(text).toBe("Three marks.");
    expect(turn.calls).toHaveLength(1);
    expect(turn.calls[0]).toMatchObject({
      tool: "marks.list",
      allowed: true,
      summary: "3 marks",
      ms: 12,
    });
    expect(turn.cards).toEqual([{ type: "marks", ids: ["a"] }]);
    expect(turn.usage).toMatchObject({ input: 800, output: 100 });
    expect(turn.usage?.costUsd).toBeCloseTo(0.0012, 6);
    expect(turn.done).toBe(true);
    expect(turn.local).toBe(false);
  });

  it("shows a refused call with its reason, and a refusal before any request", () => {
    const { turn } = fold([
      {
        type: "tool_call",
        call_id: "t1",
        tool: "manuscript.write",
        input: { text: "x" },
        allowed: false,
        reason: "Asten is your editor, not your writer.",
      },
      {
        type: "refused",
        matched: "write",
        tool: "manuscript.write",
        reply: "I can't do that one.",
      },
    ]);
    expect(turn.calls[0].allowed).toBe(false);
    expect(turn.calls[0].reason).toContain("not your writer");
    expect(turn.refused).toEqual({
      matched: "write",
      tool: "manuscript.write",
    });
  });

  it("replaces the streamed text with the final text, which may be a refusal", () => {
    const { turn, text } = fold([
      { type: "text", delta: "Once upon a time" },
      { type: "final", text: "I stopped that reply.", withheld: true },
    ]);
    expect(text).toBe("I stopped that reply.");
    expect(turn.withheld).toBe(true);
  });

  it("keeps an unpriced run unpriced", () => {
    const { turn } = fold([
      {
        type: "usage",
        input: 10,
        output: 5,
        cost_usd: null,
        estimated: true,
        totals,
      },
    ]);
    expect(turn.usage?.costUsd).toBeNull();
    expect(turn.usage?.estimated).toBe(true);
  });
});

describe("what a thread keeps", () => {
  it("drops the request bodies, which can hold a passage the writer picked", () => {
    const { turn } = fold([
      {
        type: "request",
        n: 1,
        provider: "p",
        host: "h",
        bytes: 10,
        sha256: "s",
        tokens_estimate: 3,
        body: "THE PASSAGE",
      },
    ]);
    expect(JSON.stringify(forStorage(turn))).not.toContain("THE PASSAGE");
    expect(forStorage(turn).requests[0].sha256).toBe("s");
  });
});

describe("the spend line", () => {
  it("reads in tokens and dollars and says when some calls are unpriced", () => {
    const caps = { daily_tokens: 200000, thread_tokens: 60000, daily_usd: 1 };
    expect(spendLine(totals, caps)).toBe(
      "Today 1,200 of 200,000 tokens · about $0.0123",
    );
    expect(spendLine({ ...totals, day_has_unpriced: true }, caps)).toContain(
      "unpriced",
    );
  });
});
