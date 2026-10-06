export function Download() {
  return (
    <main className="page">
      <p className="eyebrow">The library</p>
      <h1>Open it on this machine.</h1>
      <p className="lede">
        The desktop build will sit here. Until then the environment runs
        locally from the Margin repository — API on one port, the desk on
        another. Nothing cute. No account.
      </p>
      <ol className="steps">
        <li>Clone the Margin repository.</li>
        <li>
          <code>pnpm install</code>
        </li>
        <li>
          <code>pnpm dev:api</code> then <code>pnpm dev</code>
        </li>
        <li>Write a scene. Pin a scrap. Neither is the gate.</li>
      </ol>
    </main>
  );
}
