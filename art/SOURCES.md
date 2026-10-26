# Sources for the painted artwork

Everything under `apps/www/public/art/` is made by `art/build.py` from the public-domain works below. Raw
downloads stay in `art/.cache` (ignored by git); `art/sources.lock.json` holds the SHA-256 of each source file
the build used, so a later run can tell if a museum has replaced an image. No work here was drawn, generated or
redrawn: each is cropped, cut, graded and set on paper, nothing more. The same list, as a table with links, is on
`/credits`.

Run `python art/build.py` (Python 3.10+, Pillow, NumPy, opencv-python, curl). Options: `python art/build.py hero thoth`
builds only those assets; `ASTEN_ART_CACHE=/some/dir` moves the download cache.

## Works

| Key | Work | Maker and date | Holding institution and object | Licence | Source |
| --- | --- | --- | --- | --- | --- |
| `kensett` | *Sunset Sky*, oil on canvas | John Frederick Kensett, 1872 | The Metropolitan Museum of Art, 74.30 (object 11326) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/11326 |
| `sargent` | *Cliffs at Deir el Bahri, Egypt*, oil on canvas | John Singer Sargent, 1890–91 | The Metropolitan Museum of Art, 50.130.24 (object 12061) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/12061 |
| `denis` | *Cloud Study (Early Evening)*, oil on paper | Simon Denis, about 1786–1806 | The Metropolitan Museum of Art, 2003.42.18 (object 438642) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/438642 |
| `menna` | *Harvest Scenes, Tomb of Menna*, tempera on paper (facsimile) | Charles K. Wilkinson, 1930, after a tomb painting of about 1396–1347 BCE | The Metropolitan Museum of Art, 30.4.44 (object 548574) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/548574 |
| `palette` | *Scribe's Palette and Brushes*, wood, ink, reed | Egyptian, Late Period, 664–332 BCE | The Metropolitan Museum of Art, 14.1.393a–d (object 544302) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/544302 |
| `statue` | *Haremhab as a Scribe of the King*, granodiorite | Egyptian, New Kingdom, about 1328–1314 BCE | The Metropolitan Museum of Art, 23.10.1 (object 544692) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/544692 |
| `lamp` | *Bronze lamp* | Roman, about 4th century CE | The Metropolitan Museum of Art, 01.17.1 (object 246975) | CC0 (Open Access) | https://www.metmuseum.org/art/collection/search/246975 |
| `hunefer` | *The judgement of the dead in the presence of Osiris*, from the Book of the Dead of Hunefer, painted papyrus | Hunefer, about 1275 BCE | The British Museum, EA 9901; photograph on Wikimedia Commons | Public domain (Commons: PD-Art) | https://commons.wikimedia.org/wiki/File:The_judgement_of_the_dead_in_the_presence_of_Osiris.jpg |
| `ibis` | *Ibis blanc ou sacré (Ibis religiosa); Ibis noir (Ibis falcinellus)*, plate 7 of *Histoire naturelle, Oiseaux*, in the *Description de l'Égypte*, engraving | after Jules-César Savigny, Imprimerie impériale, Paris, 1809 | scan by the New York Public Library (b14212718-1268479), via Wikimedia Commons | Public domain | https://commons.wikimedia.org/wiki/File:Zoologie._Oiseau._1._Ibis_blanc_ou_sacr%C3%A9_(Ibis_religiosa);_2._Ibis_noir_(Ibis_falcinellus)_(NYPL_b14212718-1268479).jpg |

Met Open Access images are released by the museum under Creative Commons Zero; the API field `isPublicDomain` is
checked before any Met download (`fetch` in `build.py` stops if it is false).

## Fonts

| Font | Copyright | Licence |
| --- | --- | --- |
| Caveat (the margin hand) | © 2014 The Caveat Project Authors, Impallari Type for Google Fonts | SIL Open Font License 1.1 |

The other fonts (Fraunces, Literata, IBM Plex Mono, Noto Sans Egyptian Hieroglyphs) are listed on `/credits`.

## What the build does to each work

One grade for the whole set (`grade()` in `build.py`): the highlights are gained toward a warm paper light, saturation
is eased to 0.85 to 0.95, the blacks are lifted a little (a matte), and the cast is warmed. The goal is one lighting
temperature across a Hudson River sky, a New Kingdom papyrus, a Roman lamp and an 1809 engraving.

| Output (`apps/www/public/art/`) | From | Treatment |
| --- | --- | --- |
| `hero-window.webp` | `kensett`, `sargent` | A portrait slice of the sky down past its gold band; Sargent's cliffs with their own sky removed (flood-filled from the top plus one hand-drawn polygon over the hazy valley, edge feathered 2.6 px); a horizon glow and a low vignette; a quadrant-mean (Kuwahara) smoothing at radius 2 blended 55% with the original, so the brushwork stays; grade; fine grain. |
| `thoth.webp` | `hunefer` | Thoth is cut along his own ink outline: pixels close to the papyrus ground, not enclosed by a dark outline and connected to the border, are background; the gap between the legs and the neighbouring figures are removed with hand-placed polygons; the figure is enlarged 2.2x, smoothed (Kuwahara radius 2) and re-sharpened on the ink line; feathered 1.5 px and shrunk 1 px so no ground colour rims the edge. |
| `plate-scribes.webp` | `menna` | Crop of the surveyors and the boys with writing boards; light Kuwahara; grade; torn paper edge (below). |
| `plate-weighing.webp` | `hunefer` | Crop of the scales, Anubis, Thoth and the scribe Hunefer; light Kuwahara; grade; torn edge. |
| `plate-dusk.webp` | `denis` | Crop of the cloud study; grade; torn edge. |
| `plate-cliffs.webp` | `sargent` | The painting inside its border; grade; torn edge. |
| `plate-palette.webp`, `plate-lamp.webp`, `plate-scribe.webp` | `palette`, `lamp`, `statue` | Studio photographs on a grey sweep. The backdrop is modelled per row from the left and right margins, GrabCut settles the uncertain band, and the largest component is kept; the matte is feathered and shrunk 2 px; Kuwahara radius 2 blended 60% so the photograph sits with the paintings; grade (warmer for the granite). The museum's plinth under the statue is cut off. |
| `plate-ibis.webp` | `ibis` | The black ibis is removed by painting over the largest dark connected component with the plate's own margin colour; the engraving is re-toned: ink to a warm brown, paper to the site's cream; torn edge. |
| `grain.webp` | none (procedural) | A 128 px seamless paper-fibre tile: four octaves of Gaussian noise and seventy short fibres, dark specks and pale fibres as a low-alpha overlay. |

Torn edge (`torn()`): each side of the rectangle wanders by fractal noise (amplitude 12 px, a second fine octave for
fibres) and the last pixels of the edge are blended toward bare paper. Seeds are fixed, so the same tear comes out
every build.
