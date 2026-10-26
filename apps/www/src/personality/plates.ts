/**
 * The painted plates. One entry per work: the file the build made (art/build.py), its size, and the museum
 * label, which is also the credit. Sources, object numbers and licences are in art/SOURCES.md and shown in full
 * on /credits. Every work is in the public domain or released under CC0 by the holding museum.
 */

export type PlateKey = "scribes" | "palette" | "weighing" | "dusk" | "cliffs" | "lamp" | "scribe" | "ibis";

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

export const works: Record<string, Work> = {
  kensett: {
    title: "Sunset Sky",
    by: "John Frederick Kensett",
    date: "1872",
    medium: "oil on canvas",
    where: "The Metropolitan Museum of Art, 74.30",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/11326",
  },
  sargent: {
    title: "Cliffs at Deir el Bahri, Egypt",
    by: "John Singer Sargent",
    date: "1890–91",
    medium: "oil on canvas",
    where: "The Metropolitan Museum of Art, 50.130.24",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/12061",
  },
  denis: {
    title: "Cloud Study (Early Evening)",
    by: "Simon Denis",
    date: "about 1786–1806",
    medium: "oil on paper",
    where: "The Metropolitan Museum of Art, 2003.42.18",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/438642",
  },
  menna: {
    title: "Harvest Scenes, Tomb of Menna",
    by: "facsimile by Charles K. Wilkinson, after a tomb painting of about 1396–1347 BCE",
    date: "1930",
    medium: "tempera on paper",
    where: "The Metropolitan Museum of Art, 30.4.44",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/548574",
  },
  palette: {
    title: "Scribe’s Palette and Brushes",
    by: "Egyptian, Late Period",
    date: "664–332 BCE",
    medium: "wood, ink, reed",
    where: "The Metropolitan Museum of Art, 14.1.393a–d",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/544302",
  },
  statue: {
    title: "Haremhab as a Scribe of the King",
    by: "Egyptian, New Kingdom",
    date: "about 1328–1314 BCE",
    medium: "granodiorite",
    where: "The Metropolitan Museum of Art, 23.10.1",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/544692",
  },
  lamp: {
    title: "Bronze lamp",
    by: "Roman",
    date: "about 4th century CE",
    medium: "bronze",
    where: "The Metropolitan Museum of Art, 01.17.1",
    licence: "Open Access, CC0",
    href: "https://www.metmuseum.org/art/collection/search/246975",
  },
  hunefer: {
    title: "The judgement of the dead in the presence of Osiris, from the Book of the Dead of Hunefer",
    by: "Hunefer, scribe",
    date: "about 1275 BCE",
    medium: "painted papyrus",
    where: "The British Museum, EA 9901 (photograph via Wikimedia Commons)",
    licence: "Public domain",
    href: "https://commons.wikimedia.org/wiki/File:The_judgement_of_the_dead_in_the_presence_of_Osiris.jpg",
  },
  ibis: {
    title: "Ibis blanc ou sacré (Ibis religiosa), plate 7 of Histoire naturelle, Oiseaux",
    by: "after Jules-César Savigny, in the Description de l’Égypte",
    date: "1809",
    medium: "engraving",
    where: "Imprimerie impériale, Paris (scan: New York Public Library, via Wikimedia Commons)",
    licence: "Public domain",
    href: "https://digitalcollections.nypl.org/items/510d47e0-bda9-a3d9-e040-e00a18064a99",
  },
};

export interface PlateSpec {
  src: string;
  width: number;
  height: number;
  /** What a person who cannot see it needs to know. */
  alt: string;
  /** Short label shown under the plate: which work, in a line. */
  label: string;
  works: (keyof typeof works)[];
}

export const plates: Record<PlateKey, PlateSpec> = {
  scribes: {
    src: "/art/plate-scribes.webp",
    width: 1200,
    height: 583,
    alt: "Surveyors stretch a measuring cord across a field of ripe wheat while two boys carry writing boards, painted on a tomb wall in Thebes.",
    label: "Measuring the harvest, Tomb of Menna",
    works: ["menna"],
  },
  palette: {
    src: "/art/plate-palette.webp",
    width: 1100,
    height: 584,
    alt: "A scribe’s wooden palette with its black ink cake, and three reed pens lying beside it.",
    label: "A scribe’s palette and pens",
    works: ["palette"],
  },
  weighing: {
    src: "/art/plate-weighing.webp",
    width: 960,
    height: 592,
    alt: "The weighing of the heart on a papyrus: Anubis at the scales, and ibis-headed Thoth with palette and reed, recording the verdict.",
    label: "The weighing of the heart, Papyrus of Hunefer",
    works: ["hunefer"],
  },
  dusk: {
    src: "/art/plate-dusk.webp",
    width: 960,
    height: 823,
    alt: "An oil study of evening cloud, white and grey masses lit pink from below.",
    label: "Evening cloud, an oil study",
    works: ["denis"],
  },
  cliffs: {
    src: "/art/plate-cliffs.webp",
    width: 1000,
    height: 534,
    alt: "Golden limestone cliffs above a pale valley, painted in oil at Deir el-Bahri.",
    label: "The cliffs at Deir el-Bahri",
    works: ["sargent"],
  },
  lamp: {
    src: "/art/plate-lamp.webp",
    width: 760,
    height: 468,
    alt: "A green bronze oil lamp with a lid and a long handle.",
    label: "A bronze lamp",
    works: ["lamp"],
  },
  scribe: {
    src: "/art/plate-scribe.webp",
    width: 560,
    height: 828,
    alt: "A seated scribe in granite with a papyrus unrolled across his lap.",
    label: "Haremhab as a scribe",
    works: ["statue"],
  },
  ibis: {
    src: "/art/plate-ibis.webp",
    width: 940,
    height: 1060,
    alt: "An engraving of the sacred ibis, a tall white bird with a curved black bill, on old paper.",
    label: "The sacred ibis, an engraved plate",
    works: ["ibis"],
  },
};
