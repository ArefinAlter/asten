import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useEffect, useState } from "react";
import type { PairingStart } from "@asten/contracts";
import { Glyph } from "../brand/Glyphs";
import { createAstenClient, isTauri } from "../lib/asten-client";
import { timeAgo } from "../lib/format";

/** Word, Docs and browsers are guests. They pair once with a code shown here. */
export function Guests() {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const status = useQuery({
    queryKey: ["auth-status-panel"],
    queryFn: () => client.authStatus(),
  });
  const tokens = useQuery({
    queryKey: ["tokens"],
    queryFn: () => client.listTokens(),
  });
  const [code, setCode] = useState<PairingStart>();
  const [remaining, setRemaining] = useState(0);
  const [origin, setOrigin] = useState("");

  const me = useQuery({
    queryKey: ["auth-me-guests"],
    queryFn: () => client.me(),
  });

  useEffect(() => {
    client
      .origin()
      .then((value) => setOrigin(value || window.location.origin))
      .catch(() => setOrigin(window.location.origin));
  }, [client]);

  useEffect(() => {
    if (!code) return;
    const timer = window.setInterval(() => {
      const seconds = Math.max(
        0,
        Math.round((new Date(code.expires_at).getTime() - Date.now()) / 1000),
      );
      setRemaining(seconds);
      if (seconds === 0) setCode(undefined);
    }, 1000);
    return () => window.clearInterval(timer);
  }, [code]);

  const mint = useMutation({
    mutationFn: (input: { label: string; grants: "guest" | "desk" }) =>
      client.startPairing(input.label, input.grants),
    onSuccess: setCode,
  });
  const revoke = useMutation({
    mutationFn: (id: string) => client.revokeToken(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tokens"] }),
  });
  const setupOwner = useMutation({
    mutationFn: (passphrase: string) => client.setupOwner(passphrase),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["auth-status-panel"] }),
  });

  function submitMint(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const grants = form.get("grants") === "desk" ? "desk" : "guest";
    mint.mutate({
      label: String(
        form.get("label") ?? (grants === "desk" ? "Another desk" : "A guest"),
      ),
      grants,
    });
  }

  function submitOwner(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setupOwner.mutate(String(form.get("passphrase") ?? ""));
    event.currentTarget.reset();
  }

  const apiOrigin = origin || "http://127.0.0.1:4317";
  return (
    <section className="guests register">
      <header>
        <p className="eyebrow mono">Guests</p>
        <h1>Word and Docs remain guests</h1>
        <p className="lede">
          The desk answers only to clients you have paired. A guest types a
          six-digit code once and receives a token; you can revoke it here any
          time. Nothing leaves this machine.
        </p>
      </header>

      <div className="guest-columns">
        <div>
          <h2>Pair a guest</h2>
          <form className="row" onSubmit={submitMint}>
            <input
              name="label"
              placeholder="Word on this laptop"
              defaultValue="A guest"
            />
            <select
              name="grants"
              defaultValue="guest"
              aria-label="What the code grants"
            >
              <option value="guest">A guest: reads and writes the story</option>
              <option value="desk">Another desk: may also manage guests</option>
            </select>
            <button
              type="submit"
              className="btn btn--small btn--primary"
              disabled={mint.isPending || status.data?.mode === "off"}
            >
              Show a code
            </button>
          </form>
          {status.data?.mode === "off" && (
            <p className="hint">
              Authentication is off (ASTEN_AUTH=off). Every local page can reach
              the API. Use pairing outside development.
            </p>
          )}
          {code && (
            <div className="pairing-code">
              <span className="code mono">
                {code.code.slice(0, 3)} {code.code.slice(3)}
              </span>
              <span className="hint">
                for {code.label} ({code.grants}) · expires in{" "}
                {Math.floor(remaining / 60)}:
                {String(remaining % 60).padStart(2, "0")}
              </span>
            </div>
          )}
          <h3 className="mono">Where guests connect</h3>
          <p className="hint">
            API at <code>{apiOrigin}</code>. The Word task pane and the Docs
            sidebar enter the code when they first open. A browser opens{" "}
            <code>{isTauri() ? apiOrigin : window.location.origin}</code> and
            enters it on the pairing screen.
          </p>
          <ul className="pair-notes">
            <li>
              <Glyph name="door" /> Codes work once and expire in ten minutes.
              Five wrong tries lock a code.
            </li>
            <li>
              <Glyph name="eye" /> Only a hash of each token is stored. Revoking
              takes effect immediately.
            </li>
          </ul>
        </div>
        <div>
          <h2>Paired clients</h2>
          <ul className="plain-list">
            {(tokens.data ?? []).map((token) => (
              <li key={token.id} className="row">
                <span>
                  <strong>{token.label}</strong>{" "}
                  <small className="mono">{token.kind}</small>
                </span>
                <small className="hint">
                  paired {timeAgo(token.created_at)}
                  {token.last_used_at
                    ? ` · used ${timeAgo(token.last_used_at)}`
                    : ""}
                  {token.expires_at
                    ? ` · expires ${new Date(token.expires_at).toLocaleDateString()}`
                    : ""}
                </small>
                {me.data?.token_id === token.id ? (
                  <small className="hint">this desk</small>
                ) : (
                  <button
                    type="button"
                    className="linkish rubric"
                    onClick={() => revoke.mutate(token.id)}
                  >
                    revoke
                  </button>
                )}
              </li>
            ))}
            {(tokens.data ?? []).length === 0 && (
              <li className="hint">No guests yet.</li>
            )}
          </ul>
          <h2>Owner passphrase</h2>
          {status.data?.owner_configured ? (
            <p className="hint">
              Set. With ASTEN_AUTH=owner, a browser on another machine can
              unlock a session with it.
            </p>
          ) : (
            <form className="stacked-form" onSubmit={submitOwner}>
              <p className="hint">
                Optional. Only needed when you host the API for yourself beyond
                this machine (ASTEN_AUTH=owner). Twelve characters or more.
              </p>
              <input
                name="passphrase"
                type="password"
                minLength={12}
                placeholder="A long passphrase"
                required
              />
              <button
                type="submit"
                className="btn btn--small"
                disabled={setupOwner.isPending}
              >
                Set passphrase
              </button>
              {setupOwner.error && (
                <p className="error">{setupOwner.error.message}</p>
              )}
            </form>
          )}
        </div>
      </div>
    </section>
  );
}
