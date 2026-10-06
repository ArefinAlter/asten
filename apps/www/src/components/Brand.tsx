import { Link } from "react-router-dom";
import { PaletteMark } from "../art/Palette";

export function BrandLink({ onClick }: { onClick?: () => void }) {
  return (
    <Link to="/" className="brand" aria-label="Asten home" onClick={onClick}>
      <PaletteMark />
      <span className="wordmark">Asten</span>
    </Link>
  );
}

export function CartoucheWordmark({ className }: { className?: string }) {
  return (
    <span className={`cartouche ${className ?? ""}`}>
      <PaletteMark />
      <span className="wordmark">Asten</span>
    </span>
  );
}
