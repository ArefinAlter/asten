import { Link } from "react-router-dom";
import { plates, works, type PlateKey } from "./plates";

interface PlateProps {
  name: PlateKey;
  className?: string;
  /** Load straight away (above the fold). Everything else loads lazily. */
  eager?: boolean;
  /** Show the museum label under the plate. */
  label?: boolean;
  /** Lean the plate a few degrees, the way a loose sheet lies on a desk. */
  tilt?: number;
  sizes?: string;
}

/**
 * A painted plate lying on the paper, with a museum label that is also its credit. The image carries its own
 * torn edge or cut-out; the shadow, tilt and label are the page's.
 */
export function Plate({ name, className, eager = false, label = true, tilt = 0, sizes }: PlateProps) {
  const spec = plates[name];
  const credit = spec.works.map((key) => works[key]);
  return (
    <figure className={`plate plate--${name}${className ? ` ${className}` : ""}`} style={tilt ? ({ "--tilt-plate": `${tilt}deg` } as React.CSSProperties) : undefined}>
      <img
        src={spec.src}
        width={spec.width}
        height={spec.height}
        alt={spec.alt}
        loading={eager ? "eager" : "lazy"}
        decoding="async"
        sizes={sizes}
      />
      {label ? (
        <figcaption className="plate-label">
          <span className="plate-label-title">{spec.label}</span>
          <span className="plate-label-credit">
            {credit.map((work, i) => (
              <span key={work.title}>
                {i ? "; " : ""}
                <cite>{shortTitle(work.title)}</cite>, {work.by}, {work.date}. {work.where.split(" (")[0]}.
              </span>
            ))}{" "}
            <Link to="/credits">Credits</Link>
          </span>
        </figcaption>
      ) : null}
    </figure>
  );
}

function shortTitle(title: string): string {
  return title.length > 64 ? `${title.slice(0, title.indexOf(",", 24) > 0 ? title.indexOf(",", 24) : 60)}` : title;
}
