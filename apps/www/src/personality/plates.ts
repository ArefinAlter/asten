/**
 * The painted plates. art/build.py reads art/works.json (what each work is, who holds it, under what licence)
 * and art/plates.json (what to cut from it, and which page it belongs to), makes the WebP files, and writes
 * art-manifest.json beside this file. Nothing here is typed by hand: the museum label under every plate, the
 * credit line in it and the table on /credits all come from the same two files, so they cannot drift.
 */
import manifest from "./art-manifest.json";

export interface Work {
  /** Title as the museum gives it. */
  title: string;
  /** Maker, with the role when it is not painter. */
  by: string;
  date: string;
  medium?: string;
  /** Holding institution and object number. */
  where: string;
  /** What it is in the public domain under. */
  licence: string;
  href: string;
}

export type PlateShape = "torn" | "arch" | "band" | "cut";

export interface PlateSpec {
  src: string;
  width: number;
  height: number;
  bytes: number;
  shape: PlateShape;
  /** Keys into `works`. */
  works: string[];
  /** Short caption shown first on the museum label. */
  label: string;
  /** What a person who cannot see it needs to know. */
  alt: string;
}

export interface PagePlate {
  plate: string;
  /** Which side of the heading the plate lies on, wide screens only. */
  side?: "left" | "right";
}

export type PlateKey = string;

export const works = manifest.works as Record<string, Work>;
export const plates = manifest.plates as unknown as Record<string, PlateSpec>;
export const pagePlates = manifest.pages as Record<string, PagePlate>;

/** The plate that belongs to a page, if any. */
export function plateForPath(path: string): PagePlate | undefined {
  const clean = path.split(/[?#]/)[0] ?? path;
  return pagePlates[clean.length > 1 ? clean.replace(/\/+$/, "") : clean];
}

/** Every work shown anywhere on the site, in the order of its first plate, for the credits table. */
export function creditedWorks(): { key: string; work: Work; plates: string[] }[] {
  const used = new Map<string, string[]>();
  for (const [key, spec] of Object.entries(plates)) {
    for (const w of spec.works) used.set(w, [...(used.get(w) ?? []), key]);
  }
  // the hero window and the cut-out Thoth are not plates, but they are made from these works
  for (const w of ["kensett", "sargent", "hunefer"]) if (!used.has(w)) used.set(w, []);
  return [...used.entries()].map(([key, list]) => ({ key, work: works[key]!, plates: list }));
}
