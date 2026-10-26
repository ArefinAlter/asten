import type { CSSProperties } from "react";
import { Link, useLocation } from "react-router-dom";
import { plateForPath, plates, works, type PlateKey } from "./plates";

interface PlateProps {
  name: PlateKey;
  className?: string;
  /** Load straight away (above the fold). Everything else loads lazily. */
  eager?: boolean;
  /** Show the museum label under the plate. */
  label?: boolean;
  /** Lean the plate a few degrees, the way a loose sheet lies on a desk. */
  tilt?: number;
}

/**
 * A painted plate lying on the paper, with a museum label that is also its credit. The image carries its own
 * torn edge, arch or cut-out; the shadow, tilt and label are the page's.
 */
export function Plate({ name, className, eager = false, label = true, tilt = 0 }: PlateProps) {
  const spec = plates[name];
  if (!spec) throw new Error(`No plate called "${name}" in art-manifest.json`);
  const credit = spec.works.map((key) => works[key]!);
  return (
    <figure
      className={`plate plate--${spec.shape} plate--${name}${className ? ` ${className}` : ""}`}
      style={{ "--ar": spec.width / spec.height, ...(tilt ? { "--tilt-plate": `${tilt}deg` } : {}) } as CSSProperties}
    >
      <img
        src={spec.src}
        width={spec.width}
        height={spec.height}
        alt={spec.alt}
        loading={eager ? "eager" : "lazy"}
        fetchPriority={eager ? "high" : undefined}
        decoding="async"
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

/** The plate that belongs to a path (a page, a journal post); a thumbnail with no label when `bare`. */
export function PagePlate({ path, bare = false, className, eager = false }: { path: string; bare?: boolean; className?: string; eager?: boolean }) {
  const pp = plateForPath(path);
  if (!pp) return null;
  return <Plate name={pp.plate} className={`${bare ? "plate--thumb " : ""}${className ?? ""}`.trim()} label={!bare} eager={eager} />;
}

/** The plate for the page being shown. */
export function CurrentPlate({ className }: { className?: string }) {
  const { pathname } = useLocation();
  return <PagePlate path={pathname} className={className} />;
}

function shortTitle(title: string): string {
  if (title.length <= 64) return title;
  const cut = title.indexOf(",", 24);
  return cut > 0 ? title.slice(0, cut) : `${title.slice(0, 60)}…`;
}
