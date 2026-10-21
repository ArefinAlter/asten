import { type FormEvent, useEffect, useRef, useState } from "react";
import type { AuthStatus } from "@asten/contracts";
import { PaletteMark } from "../brand/Palette";
import { Glyph } from "../brand/Glyphs";
import { createAstenClient } from "../lib/asten-client";

interface Props {
  status: AuthStatus;
  onPaired: () => void;
}

const ERROR_ID = "pair-error";

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
  const codeRef = useRef<HTMLInputElement>(null);
  const passphraseRef = useRef<HTMLInputElement>(null);

  // Focus lands on the secret field when the screen opens or the mode changes.
  useEffect(() => {
    (mode === "code" ? codeRef : passphraseRef).current?.focus();
  }, [mode]);

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

  const describedBy = error ? ERROR_ID : undefined;
  const invalid = error ? true : undefined;

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
          <div className="segmented" role="tablist" aria-label="How to unlock">
            <button
              type="button"
              role="tab"
              id="pair-tab-code"
              aria-selected={mode === "code"}
              aria-controls="pair-form"
              className={mode === "code" ? "active" : ""}
              onClick={() => setMode("code")}
            >
              Pairing code
            </button>
            <button
              type="button"
              role="tab"
              id="pair-tab-passphrase"
              aria-selected={mode === "passphrase"}
              aria-controls="pair-form"
              className={mode === "passphrase" ? "active" : ""}
              onClick={() => setMode("passphrase")}
              disabled={!status.owner_configured}
            >
              Owner passphrase
            </button>
          </div>
        )}
        <form
          id="pair-form"
          className="stacked-form pair-form"
          onSubmit={submit}
          aria-labelledby={
            status.mode === "owner" ? `pair-tab-${mode}` : undefined
          }
        >
          {mode === "code" ? (
            <>
              <label htmlFor="pair-code">
                <span>Pairing code</span>
                <input
                  id="pair-code"
                  ref={codeRef}
                  name="code"
                  className="code-input"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  pattern="[0-9 ]*"
                  maxLength={7}
                  placeholder="000 000"
                  value={code}
                  onChange={(event) => setCode(event.target.value)}
                  aria-describedby={describedBy}
                  aria-invalid={invalid}
                  required
                />
              </label>
              <label htmlFor="pair-label">
                <span>Call this client</span>
                <input
                  id="pair-label"
                  name="label"
                  autoComplete="off"
                  value={label}
                  onChange={(event) => setLabel(event.target.value)}
                />
              </label>
            </>
          ) : (
            <label htmlFor="pair-passphrase">
              <span>Owner passphrase</span>
              <input
                id="pair-passphrase"
                ref={passphraseRef}
                name="passphrase"
                type="password"
                autoComplete="current-password"
                value={passphrase}
                onChange={(event) => setPassphrase(event.target.value)}
                aria-describedby={describedBy}
                aria-invalid={invalid}
                required
              />
            </label>
          )}
          {error && (
            <p id={ERROR_ID} className="error" role="alert">
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
