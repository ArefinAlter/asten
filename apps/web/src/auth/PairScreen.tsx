import { type FormEvent, useState } from "react";
import type { AuthStatus } from "@asten/contracts";
import { PaletteMark } from "../brand/Palette";
import { Glyph } from "../brand/Glyphs";
import { createAstenClient } from "../lib/asten-client";

interface Props {
  status: AuthStatus;
  onPaired: () => void;
}

/**
 * The browser desk pairs with the API once. The code is printed by the API
 * when it starts with no paired client, or minted in the Guests panel of a
 * paired desk. In owner mode a passphrase opens a session instead.
 */
export function PairScreen({ status, onPaired }: Props) {
  const client = createAstenClient();
  const [mode, setMode] = useState<"code" | "passphrase">(
    status.mode === "owner" && status.owner_configured ? "passphrase" : "code",
  );
  const [code, setCode] = useState("");
  const [passphrase, setPassphrase] = useState("");
  const [label, setLabel] = useState("This browser");
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    try {
      if (mode === "code") await client.pair(code.replace(/\D/g, ""), label);
      else await client.login(passphrase);
      onPaired();
    } catch (failure) {
      setError(
        failure instanceof Error ? failure.message : "That did not work.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="pair-screen">
      <section className="pair-card register">
        <div className="pair-head">
          <PaletteMark />
          <h1>Pair this browser with your desk</h1>
        </div>
        <p className="lede">
          Asten keeps your manuscript on this machine and answers only to
          clients you have paired. The API printed a six-digit code when it
          started; a paired desk can mint another under <em>Guests</em>.
        </p>
        {status.mode === "owner" && (
          <div className="segmented" role="tablist">
            <button
              type="button"
              role="tab"
              aria-selected={mode === "code"}
              className={mode === "code" ? "active" : ""}
              onClick={() => setMode("code")}
            >
              Pairing code
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={mode === "passphrase"}
              className={mode === "passphrase" ? "active" : ""}
              onClick={() => setMode("passphrase")}
              disabled={!status.owner_configured}
            >
              Owner passphrase
            </button>
          </div>
        )}
        <form className="stacked-form pair-form" onSubmit={submit}>
          {mode === "code" ? (
            <>
              <label>
                <span>Pairing code</span>
                <input
                  className="code-input"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  placeholder="000 000"
                  value={code}
                  onChange={(event) => setCode(event.target.value)}
                  autoFocus
                  required
                />
              </label>
              <label>
                <span>Call this client</span>
                <input
                  value={label}
                  onChange={(event) => setLabel(event.target.value)}
                />
              </label>
            </>
          ) : (
            <label>
              <span>Owner passphrase</span>
              <input
                type="password"
                value={passphrase}
                onChange={(event) => setPassphrase(event.target.value)}
                autoFocus
                required
              />
            </label>
          )}
          {error && (
            <p className="error" role="alert">
              {error}
            </p>
          )}
          <button type="submit" className="btn btn--primary" disabled={busy}>
            {busy ? "Pairing…" : mode === "code" ? "Pair" : "Unlock"}
          </button>
        </form>
        <ul className="pair-notes">
          <li>
            <Glyph name="house" /> Pairing is local. No account, no sync,
            nothing leaves this machine.
          </li>
          <li>
            <Glyph name="eye" /> A code works once and expires in ten minutes.
            Revoke any client from the Guests panel.
          </li>
        </ul>
      </section>
    </main>
  );
}
