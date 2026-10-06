import { Link } from "react-router-dom";
import { IbisStandard } from "../art/Figures";
import { Arrow } from "../components/Icons";

export function NotFound() {
  return (
    <section className="register not-found" aria-labelledby="nf-h">
      <div className="wrap split">
        <IbisStandard />
        <div>
          <p className="eyebrow">404</p>
          <h1 id="nf-h">Lost to the sand.</h1>
          <p className="lede" style={{ margin: "18px 0 28px" }}>
            This page isn’t in the record. It may have moved, or it may never have been written. The scribe has no note of it.
          </p>
          <div className="btn-row">
            <Link to="/" className="btn btn--primary">
              Back to the start
            </Link>
            <Link to="/features" className="link-arrow">
              Browse features <Arrow />
            </Link>
            <Link to="/journal" className="link-arrow">
              Read the journal <Arrow />
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
