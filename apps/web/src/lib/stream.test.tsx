// @vitest-environment jsdom
import {
  QueryClient,
  QueryClientProvider,
  MutationObserver,
} from "@tanstack/react-query";
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { StreamEvent } from "@asten/contracts";
import { SAVE_DOCUMENT_KEY, useProjectStream } from "./hooks";

const stream = vi.hoisted(() => ({
  event: undefined as ((event: StreamEvent) => void) | undefined,
}));
vi.mock("./asten-client", () => ({
  createAstenClient: () => ({
    subscribe: async (
      _project: string,
      callback: (event: StreamEvent) => void,
    ) => {
      stream.event = callback;
      return () => {};
    },
  }),
}));

afterEach(cleanup);

function Subscriber() {
  useProjectStream("project");
  return null;
}

it("refetches a newer remote head received while a local save was pending", async () => {
  const client = new QueryClient();
  const key = ["document", "project", "page"];
  client.setQueryData(key, { head_seq: 0 });
  const invalidate = vi.spyOn(client, "invalidateQueries");
  render(
    <QueryClientProvider client={client}>
      <Subscriber />
    </QueryClientProvider>,
  );
  let finish!: () => void;
  const mutation = new MutationObserver(client, {
    mutationKey: SAVE_DOCUMENT_KEY,
    mutationFn: (_input: { documentId: string }) =>
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
    onSuccess: () => {
      client.setQueryData(key, { head_seq: 1 });
    },
  });
  const saving = mutation.mutate({ documentId: "page" });
  await waitFor(() => expect(finish).toBeTypeOf("function"));
  stream.event!({ type: "document", document: "page", seq: 2 } as StreamEvent);
  expect(invalidate).not.toHaveBeenCalledWith({ queryKey: key });
  finish();
  await saving;
  expect(invalidate).toHaveBeenCalledWith({ queryKey: key });
  expect(client.getQueryState(key)?.isInvalidated).toBe(true);
});

it("does not refetch an acknowledged save or an older stream event", async () => {
  const client = new QueryClient();
  const key = ["document", "project", "page"];
  client.setQueryData(key, { head_seq: 2 });
  const invalidate = vi.spyOn(client, "invalidateQueries");
  render(
    <QueryClientProvider client={client}>
      <Subscriber />
    </QueryClientProvider>,
  );
  stream.event!({ type: "document", document: "page", seq: 1 } as StreamEvent);
  expect(invalidate).not.toHaveBeenCalledWith({ queryKey: key });
});
