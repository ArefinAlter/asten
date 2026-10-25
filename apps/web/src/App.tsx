import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { PairScreen } from "./auth/PairScreen";
import { PaletteMark } from "./brand/Palette";
import { ApiError, createAstenClient } from "./lib/asten-client";
import { isHosted } from "./lib/storage";
import { Workbench } from "./workbench/Workbench";

const client = createAstenClient();

export type View =
  | "overview"
  | "write"
  | "cast"
  | "canon"
  | "time"
  | "world"
  | "history"
  | "guests"
  | "settings"
  | "compile"
  | "packs";

function useAuthGate() {
  const [tick, setTick] = useState(0);
  const status = useQuery({
    queryKey: ["auth-status", tick],
    queryFn: () => client.authStatus(),
    retry: 2,
  });
  const me = useQuery({
    queryKey: ["auth-me", tick],
    queryFn: () => client.me(),
    enabled: Boolean(status.data?.requires_auth),
    retry: false,
  });
  const authenticated = status.data
    ? !status.data.requires_auth || me.isSuccess
    : false;
  const needsPairing =
    Boolean(status.data?.requires_auth) &&
    me.isError &&
    me.error instanceof ApiError &&
    me.error.status === 401;
  // A revoked or expired token surfaces as a 401 on any request; the client
  // forgets it and tells us, and we re-check so the pair screen comes back.
  useEffect(() => {
    const onUnauthorized = () => setTick((value) => value + 1);
    window.addEventListener("asten:unauthorized", onUnauthorized);
    return () =>
      window.removeEventListener("asten:unauthorized", onUnauthorized);
  }, []);
  return {
    status: status.data,
    loading:
      status.isPending || (Boolean(status.data?.requires_auth) && me.isPending),
    authenticated,
    needsPairing,
    unreachable: status.isError,
    error: status.error,
    refresh: () => setTick((value) => value + 1),
  };
}

function App() {
  const gate = useAuthGate();
  if (gate.unreachable) {
    return (
      <main className="pair-screen">
        <section className="pair-card register">
          <div className="pair-head">
            <PaletteMark />
            <h1>
              {isHosted()
                ? "The hosted pre-alpha is not available"
                : "The desk cannot reach its API"}
            </h1>
          </div>
          {isHosted() ? (
            <p className="lede">
              {gate.error instanceof ApiError && gate.error.status !== 502
                ? gate.error.message
                : "The server did not answer."}{" "}
              The desktop app runs the same workbench on your own machine.
            </p>
          ) : (
            <p className="lede">
              Start the local API (<code>pnpm dev:api</code>) or open the
              desktop app. Nothing of yours lives anywhere but this machine, so
              there is no server to wait for.
            </p>
          )}
          <button
            type="button"
            className="btn btn--primary"
            onClick={gate.refresh}
          >
            Try again
          </button>
        </section>
      </main>
    );
  }
  if (gate.loading || !gate.status) {
    return (
      <main className="pair-screen">
        <p className="status">Opening the desk…</p>
      </main>
    );
  }
  if (!gate.authenticated || gate.needsPairing) {
    return <PairScreen status={gate.status} onPaired={gate.refresh} />;
  }
  return <Workbench />;
}

export default App;
