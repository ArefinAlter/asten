import { describe, expect, it } from "vitest";
import type {
  AstenDiagnostic,
  AstenDocument,
  DocumentSummary,
} from "@asten/contracts";
import { SampleBackend, blockText } from "./backend";
import { SAMPLE_PROJECT_ID } from "./project";

const P = `/projects/${SAMPLE_PROJECT_ID}`;

describe("sample backend", () => {
  it("answers the desk's first requests with no auth and one project", async () => {
    const backend = new SampleBackend();
    expect((await backend.handle("GET", "/auth/status")).body).toMatchObject({
      requires_auth: false,
    });
    const projects = (await backend.handle("GET", "/projects")).body as {
      name: string;
    }[];
    expect(projects.map((p) => p.name)).toEqual(["Salt and Lantern"]);
    const summaries = (
      await backend.handle("GET", `${P}/documents?fields=summary`)
    ).body as DocumentSummary[];
    expect(summaries.find((d) => d.id === "sc1")?.words).toBeGreaterThan(40);
    expect(summaries.some((d) => "content_json" in d)).toBe(false);
  });

  it("locates every prepared mark in the text", async () => {
    const backend = new SampleBackend();
    const snapshot = (await backend.handle("GET", `${P}/story/diagnostics`))
      .body as { diagnostics: AstenDiagnostic[] };
    expect(snapshot.diagnostics.map((d) => d.code)).toEqual([
      "CANON-E013",
      "CANON-E201",
      "CANON-E401",
      "NAME-W003",
      "PLOT-I001",
      "CANON-E301",
    ]);
    for (const mark of snapshot.diagnostics)
      expect(mark.primary?.end).toBeGreaterThan(mark.primary?.start ?? 0);
  });

  it("drops a mark once the author changes the words it quotes", async () => {
    const backend = new SampleBackend();
    const page = (await backend.handle("GET", `${P}/documents/sc4`))
      .body as AstenDocument;
    const content = JSON.parse(JSON.stringify(page.content_json));
    const block = content.content[2];
    block.content[0].text = blockText(block).replace("brown eyes", "grey eyes");
    const saved = await backend.handle("PUT", `${P}/documents/sc4`, {
      content_json: content,
      expected_head_seq: page.head_seq,
    });
    expect(saved.status).toBe(200);
    const snapshot = (await backend.handle("GET", `${P}/story/diagnostics`))
      .body as { diagnostics: AstenDiagnostic[] };
    expect(snapshot.diagnostics.some((d) => d.code === "CANON-E013")).toBe(
      false,
    );
    const stale = await backend.handle("PUT", `${P}/documents/sc4`, {
      content_json: content,
      expected_head_seq: page.head_seq,
    });
    expect(stale.status).toBe(409);
  });

  it("applies a repair and records the version history", async () => {
    const backend = new SampleBackend();
    const applied = await backend.handle("POST", `${P}/story/repairs/apply`, {
      diagnostic_id: "d4",
      repair_index: 0,
    });
    expect(applied.status).toBe(200);
    const entities = (await backend.handle("GET", `${P}/entities`)).body as {
      id: string;
      aliases: string[];
    }[];
    expect(entities.find((e) => e.id === "e-ida")?.aliases).toContain("Ada");
    const snap = await backend.handle("POST", `${P}/documents/sc2/snapshots`, {
      label: "Tighten the ferry",
    });
    expect(snap.status).toBe(200);
    expect(
      (
        (await backend.handle("GET", `${P}/documents/sc2/snapshots`))
          .body as unknown[]
      ).length,
    ).toBe(2);
  });

  it("compiles text formats here and says where EPUB is written", async () => {
    const backend = new SampleBackend();
    const md = await backend.handle("POST", `${P}/compile`, {
      format: "markdown",
    });
    expect((md.body as { body: string }).body).toMatch(/^# Salt and Lantern/);
    const epub = await backend.handle("POST", `${P}/compile`, {
      format: "epub",
    });
    expect(epub.status).toBe(501);
  });
});
