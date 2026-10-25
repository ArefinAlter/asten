import { Link } from "react-router-dom";
import { IbisStandard } from "../art/Figures";
import { Arrow } from "../components/Icons";

export function NotFound() {
  return (
    <section className="register not-found" aria-labelledby="nf-h">
      <div className="wrap split">
        <IbisStandard />
        <div>
          <h1 id="nf-h">Page not found</h1>
          <p className="lede" style={{ margin: "18px 0 28px" }}>
            There is no page at this address. It may have moved.
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
