import { Reveal } from "../components/Reveal";

export function LocalPage() {
  return (
    <main className="page">
      <p className="eyebrow">Ownership</p>
      <h1>The files stay on this machine.</h1>
      <p className="lede">
        Desktop first. Offline first. Optional AI sees a pack you chose,
        never the whole drawer. A cloud lock on unpublished work is a
        product failure.
      </p>
      <Reveal className="cards">
        <article>
          <h2>SQLite, on disk</h2>
          <p>Projects, scenes, facts, and the wall live in a local database you can copy.</p>
        </article>
        <article>
          <h2>Export is not a favor</h2>
          <p>If Margin vanished tomorrow, the manuscript still opens elsewhere.</p>
        </article>
        <article>
          <h2>Provenance</h2>
          <p>
            Stable nodes, an append-only history, a source for every settled
            fact. Character offsets are not identities.
          </p>
        </article>
      </Reveal>
    </main>
  );
}
