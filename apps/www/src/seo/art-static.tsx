import { renderToStaticMarkup } from "react-dom/server";
import { IbisStandard, Lotus, Wedjat } from "../art/Figures";
import { LunarEmblem } from "../art/Night";
import { PaletteMark } from "../art/Palette";
import { PyramidSection } from "../art/Pyramid";
import { Scales } from "../art/Scales";
import { Thoth } from "../art/Thoth";
import { ScrollArt } from "../components/Page";
import { articles } from "../content/journal";
import { audiences } from "../content/audiences";
import { features } from "../content/features";
import type { ArtKey } from "../content/types";
import type { RouteMeta } from "../routes";

export type OgArt = ArtKey | "thoth";

/** Art as static SVG markup with class names; the prerender script inlines the colours. */
export function artMarkup(key: OgArt): string {
  const node = (() => {
    switch (key) {
      case "thoth":
        return <Thoth />;
      case "scales":
        return <Scales tilt={-8} />;
      case "moon":
        return <LunarEmblem />;
      case "pyramid":
        return <PyramidSection />;
      case "eye":
        return <Wedjat />;
      case "palette":
        return <PaletteMark />;
      case "ibis":
        return <IbisStandard />;
      case "lotus":
        return <Lotus />;
      case "scroll":
        return <ScrollArt />;
    }
  })();
  return renderToStaticMarkup(node);
}

export function ogArtFor(meta: RouteMeta): OgArt {
  const feature = features.find((item) => item.path === meta.path);
  if (feature) return feature.art;
  const audience = audiences.find((item) => item.path === meta.path);
  if (audience) return audience.art;
  const article = articles.find((item) => `/journal/${item.slug}` === meta.path);
  if (article) return article.art;
  switch (meta.path) {
    case "/":
    case "/ai":
    case "/name":
      return "thoth";
    case "/how-it-works":
    case "/compare":
      return "scales";
    case "/roadmap":
      return "moon";
    case "/download":
    case "/brand":
    case "/guide":
      return "palette";
    case "/privacy":
    case "/cookies":
      return "eye";
  }
  if (meta.kind === "legal") return "scales";
  if (meta.kind === "learn" || meta.kind === "journal" || meta.kind === "error") return "ibis";
  if (meta.kind === "company") return "lotus";
  return "scales";
}
