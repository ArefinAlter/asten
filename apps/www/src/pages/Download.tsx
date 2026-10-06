import { Link } from "react-router-dom";

export function Download() {
  return (
    <main className="page">
      <p className="eyebrow">The library</p>
      <h1>Open it on this machine.</h1>
      <p className="lede">
        The desktop build will sit here. Until then Asten runs locally from
        the repository — API on one port, the desk on another. Nothing cute.
        No account.
      </p>
      <ol className="steps">
        <li>Clone the Asten repository.</li>
        <li>
          <code>pnpm install</code>
        </li>
        <li>
          <code>pnpm dev:api</code> then <code>pnpm dev</code>
        </li>
        <li>Write a scene. Pin a scrap. Neither is the gate.</li>
      </ol>
      <p>
        Asten is a Verne product. Read the <Link to="/license">license</Link>{" "}
        and the <Link to="/privacy">privacy policy</Link> before you treat a
        diagnostic as a verdict.
      </p>
    </main>
  );
}
