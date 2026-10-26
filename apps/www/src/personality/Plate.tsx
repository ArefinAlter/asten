import type { CSSProperties } from "react";
import { useLocation } from "react-router-dom";
import { plateForPath, plates, type PlateKey } from "./plates";

interface PlateProps {
  name: PlateKey;
  className?: string;
  /** Load straight away (above the fold). Everything else loads lazily. */
  eager?: boolean;
  /** Lean the plate a few degrees, the way a loose sheet lies on a desk. */
  tilt?: number;
}

/**
 * A painted plate lying on the paper. The image carries its own torn edge, arch or cut-out; the shadow and
 * the tilt are the page's. There is no caption: the credit for every work is on /credits, and the picture
 * has alt text that says what it shows.
 */
export function Plate({ name, className, eager = false, tilt = 0 }: PlateProps) {
  const spec = plates[name];
  if (!spec) throw new Error(`No plate called "${name}" in art-manifest.json`);
  const style = { "--ar": spec.width / spec.height, ...(spec.focus ? { "--focus": spec.focus } : {}), ...(tilt ? { "--tilt-plate": `${tilt}deg` } : {}) } as CSSProperties;
  return (
    <figure className={`plate plate--${spec.shape} plate--${name}${className ? ` ${className}` : ""}`} style={style}>
      <img
        src={spec.src}
        width={spec.width}
        height={spec.height}
        alt={spec.alt}
        loading={eager ? "eager" : "lazy"}
        fetchPriority={eager ? "high" : undefined}
        decoding="async"
      />
    </figure>
  );
}

/** The plate that belongs to a path (a page, a journal post); `bare` fills a card slot. */
export function PagePlate({ path, bare = false, className, eager = false }: { path: string; bare?: boolean; className?: string; eager?: boolean }) {
  const pp = plateForPath(path);
  if (!pp) return null;
  return <Plate name={pp.plate} className={`${bare ? "plate--thumb " : ""}${className ?? ""}`.trim()} eager={eager} />;
}

/** The plate for the page being shown. */
export function CurrentPlate({ className }: { className?: string }) {
  const { pathname } = useLocation();
  return <PagePlate path={pathname} className={className} />;
}
