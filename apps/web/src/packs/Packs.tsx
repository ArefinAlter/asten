import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createAstenClient } from "../lib/asten-client";

/** Method packs: extensions, never a doctrine. */
export function Packs() {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const packs = useQuery({
    queryKey: ["packs"],
    queryFn: () => client.listPacks(),
  });
  const toggle = useMutation({
    mutationFn: (input: { id: string; enabled: boolean }) =>
      client.updatePack(input.id, input.enabled),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["packs"] });
      await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
    },
  });

  return (
    <section className="packs register">
      <header>
        <p className="eyebrow mono">Method packs</p>
        <h1>Extensions, not a doctrine</h1>
        <p className="lede">
          Packs add rules and optional craft hints. They never replace the page.
          Scribe stays on. Diction is a hint pack, off until you ask.
        </p>
      </header>
      <ul className="pack-list">
        {(packs.data ?? []).map((pack) => {
          const rules = ((pack.manifest as { rules?: string[] } | null)
            ?.rules ?? []) as string[];
          return (
            <li key={pack.id} className={`pack${pack.enabled ? " on" : ""}`}>
              <div>
                <strong>{pack.name}</strong>{" "}
                <small className="mono">{pack.version}</small>
                <p>{pack.summary}</p>
                <p className="codes">
                  {rules.map((code) => (
                    <code key={code} className="chip chip--code">
                      {code}
                    </code>
                  ))}
                </p>
              </div>
              <button
                type="button"
                className={`btn btn--small${pack.enabled ? "" : " btn--ghost"}`}
                disabled={pack.pack_key === "scribe" || toggle.isPending}
                onClick={() =>
                  toggle.mutate({ id: pack.id, enabled: !pack.enabled })
                }
              >
                {pack.enabled ? "On" : "Off"}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
