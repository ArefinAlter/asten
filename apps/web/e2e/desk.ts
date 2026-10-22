import type { APIRequestContext, Page } from "@playwright/test";

/** The port the e2e API listens on. 4317 is the real desk's; never that. */
export const E2E_PORT = 4391;
export const E2E_ORIGIN = `http://127.0.0.1:${E2E_PORT}`;

export interface Project {
  id: string;
  name: string;
}

export interface AstenDocument {
  id: string;
  title: string;
  head_seq: number;
}

export interface Diagnostic {
  id: string;
  code: string;
  suppressed_by?: string | null;
}

/** The desk token the global setup paired for; never logged. */
export function deskToken(): string {
  const token = process.env.ASTEN_E2E_TOKEN;
  if (!token) {
    throw new Error(
      "The e2e setup did not pair with the API. Run the suite with `pnpm e2e`.",
    );
  }
  return token;
}

/**
 * Put the setup's token where the desk's client reads it, before any page
 * script runs, so a test starts past the pair screen.
 */
export async function signIn(page: Page): Promise<void> {
  await page.addInitScript((token) => {
    try {
      window.localStorage.setItem("asten-token", token);
    } catch {
      // Storage blocked: the test will land on the pair screen and say so.
    }
  }, deskToken());
}

export function paragraph(text: string, id = "b1") {
  return {
    type: "paragraph",
    attrs: { id },
    content: text ? [{ type: "text", text }] : [],
  };
}

export function doc(...blocks: ReturnType<typeof paragraph>[]) {
  return { type: "doc", content: blocks };
}

let serial = 0;
/** A name no other test in this run uses. */
export function unique(prefix: string): string {
  serial += 1;
  return `${prefix} ${Date.now().toString(36)}${serial}`;
}

/** The API as the paired desk sees it, for arranging what a test needs. */
export class Desk {
  readonly request: APIRequestContext;
  readonly token: string;

  constructor(request: APIRequestContext, token = deskToken()) {
    this.request = request;
    this.token = token;
  }

  private async send<T>(
    method: "GET" | "POST" | "PUT",
    path: string,
    data?: unknown,
  ): Promise<T> {
    const response = await this.request.fetch(`${E2E_ORIGIN}/api/v1${path}`, {
      method,
      data,
      headers: {
        authorization: `Bearer ${this.token}`,
        accept: "application/json",
      },
    });
    if (!response.ok()) {
      throw new Error(`${method} ${path} answered ${response.status()}`);
    }
    return (await response.json()) as T;
  }

  createProject(name = unique("Project")): Promise<Project> {
    return this.send<Project>("POST", "/projects", { name, kind: "novel" });
  }

  createDocument(
    projectId: string,
    input: {
      title: string;
      kind?: "chapter" | "scene";
      text?: string;
      storyTime?: string;
      position?: number;
    },
  ): Promise<AstenDocument> {
    return this.send<AstenDocument>(
      "POST",
      `/projects/${projectId}/documents`,
      {
        project_id: projectId,
        kind: input.kind ?? "scene",
        title: input.title,
        content_json: doc(paragraph(input.text ?? "")),
        position: input.position ?? 0,
        scene: input.storyTime ? { story_time: input.storyTime } : null,
      },
    );
  }

  createEntity(projectId: string, name: string, kind = "character") {
    return this.send<{ id: string }>(
      "POST",
      `/projects/${projectId}/entities`,
      {
        project_id: projectId,
        kind,
        name,
        aliases: [],
        notes: "",
      },
    );
  }

  createFact(
    projectId: string,
    input: {
      entity: string;
      predicate: string;
      value: string;
      validFrom?: string;
    },
  ) {
    return this.send<{ id: string }>("POST", `/projects/${projectId}/facts`, {
      project_id: projectId,
      entity_key: input.entity,
      predicate: input.predicate,
      value: input.value,
      status: "confirmed",
      valid_from: input.validFrom ?? null,
    });
  }

  async diagnostics(projectId: string): Promise<Diagnostic[]> {
    const snapshot = await this.send<{ diagnostics: Diagnostic[] }>(
      "GET",
      `/projects/${projectId}/story/diagnostics`,
    );
    return snapshot.diagnostics;
  }

  /** Mint a fresh six-digit pairing code. Single use, ten minutes. */
  async mintCode(label: string, grants: "desk" | "guest" = "desk") {
    const start = await this.send<{ code: string }>(
      "POST",
      "/auth/pairing-codes",
      { label, grants },
    );
    return start.code;
  }
}

/**
 * Mira's hair was settled as auburn on day 3. Chapter 19, on day 22, may say
 * it is black: that is the CANON-E013 the Scales should raise.
 */
export async function seedMira(
  desk: Desk,
  options: { contradict: boolean },
): Promise<{
  project: Project;
  chapter2: AstenDocument;
  chapter19: AstenDocument;
}> {
  const project = await desk.createProject(unique("Marks"));
  await desk.createEntity(project.id, "Mira");
  const chapter2 = await desk.createDocument(project.id, {
    title: "Chapter 2",
    kind: "chapter",
    text: "Mira tied back her auburn hair.",
    storyTime: "day 3",
    position: 0,
  });
  await desk.createFact(project.id, {
    entity: "Mira",
    predicate: "hair_color",
    value: "auburn",
    validFrom: "day 3",
  });
  const chapter19 = await desk.createDocument(project.id, {
    title: "Chapter 19",
    kind: "chapter",
    text: options.contradict ? CONTRADICTION : "",
    storyTime: "day 22",
    position: 1,
  });
  return { project, chapter2, chapter19 };
}

export const CONTRADICTION = "Mira pushed her black hair out of her eyes.";
