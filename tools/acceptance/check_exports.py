"""Validate the files the compile step wrote.

    python check_exports.py <folder> [--expect-words N]

Every format is opened the way its own readers would: the DOCX with python-docx
(and its XML parts checked for tracked changes), the EPUB structurally (the
`mimetype` entry first and stored, container.xml, the package document's
manifest and spine, the navigation document, the NCX, every XHTML well-formed),
the FDX and the HTML as XML/HTML, Markdown, plain text and Fountain by their
markers. Prints one JSON object; exits 1 if any check failed.
"""
import json
import re
import sys
import xml.etree.ElementTree as ET
import zipfile
from html.parser import HTMLParser
from pathlib import Path

results = []


def check(name, ok, detail=""):
    results.append({"check": name, "ok": bool(ok), "detail": str(detail)[:300]})


def read_text(path):
    return Path(path).read_text(encoding="utf-8")


def words(text):
    return len(text.split())


def check_markdown(path):
    text = read_text(path)
    check("markdown: has a title and chapter headings", text.startswith("# ") and text.count("\n## ") >= 2, path.name)
    check("markdown: keeps italics", re.search(r"\*[^*\n]+\*", text) is not None)
    return words(text)


def check_plain(path):
    text = read_text(path)
    check("plain: not empty, no markup", len(text) > 100 and "<p>" not in text, path.name)
    return words(text)


def check_fountain(path):
    text = read_text(path)
    check("fountain: has a title page", text.startswith("Title:"), path.name)


class Collect(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tags = []
        self.errors = 0
        self.stack = []
        self.void = {"br", "hr", "meta", "link", "img", "input"}

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)
        if tag not in self.void:
            self.stack.append(tag)

    def handle_startendtag(self, tag, attrs):
        self.tags.append(tag)

    def handle_endtag(self, tag):
        if tag in self.void:
            return
        if self.stack and self.stack[-1] == tag:
            self.stack.pop()
        else:
            self.errors += 1


def check_html(path):
    text = read_text(path)
    parser = Collect()
    parser.feed(text)
    check("html: tags balance", parser.errors == 0 and not parser.stack, f"errors={parser.errors} open={parser.stack[:3]}")
    check("html: headings and paragraphs and emphasis", {"h1", "h2", "p", "em"} <= set(parser.tags), sorted(set(parser.tags))[:12])
    check("html: declares utf-8 and a language", "charset=\"utf-8\"" in text and "lang=\"en\"" in text)


def check_docx(path, expect_tracked=False):
    from docx import Document

    doc = Document(str(path))
    paragraphs = doc.paragraphs
    headings = [p for p in paragraphs if p.style.name.startswith("Heading") or p.style.name == "Title"]
    italics = sum(1 for p in paragraphs for r in p.runs if r.italic)
    check("docx: opens with python-docx", len(paragraphs) > 3, f"{len(paragraphs)} paragraphs")
    check("docx: has a title and headings", len(headings) >= 2, [h.style.name for h in headings[:5]])
    check("docx: keeps italics", italics >= 1, f"{italics} italic runs")
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        check("docx: core parts present", {"[Content_Types].xml", "word/document.xml", "word/styles.xml", "docProps/core.xml"} <= set(names), names)
        for name in names:
            if name.endswith(".xml") or name.endswith(".rels"):
                try:
                    ET.fromstring(z.read(name))
                except ET.ParseError as error:
                    check(f"docx: {name} is well-formed", False, error)
        xml = z.read("word/document.xml").decode("utf-8")
        if expect_tracked:
            check("docx: tracked insertions and deletions are real Word revisions", "<w:ins " in xml and "<w:del " in xml and "<w:delText" in xml)
    return words(" ".join(p.text for p in paragraphs))


def check_epub(path):
    z = zipfile.ZipFile(path)
    infos = z.infolist()
    check("epub: mimetype is the first entry", infos[0].filename == "mimetype", infos[0].filename)
    check("epub: mimetype is stored, not compressed", infos[0].compress_type == zipfile.ZIP_STORED)
    check("epub: mimetype content", z.read("mimetype") == b"application/epub+zip")
    container = ET.fromstring(z.read("META-INF/container.xml"))
    ns = {"c": "urn:oasis:names:tc:opendocument:xmlns:container"}
    rootfile = container.find(".//c:rootfile", ns).attrib["full-path"]
    check("epub: container.xml points at the package", rootfile in z.namelist(), rootfile)
    opf = ET.fromstring(z.read(rootfile))
    pkg = "{http://www.idpf.org/2007/opf}"
    manifest = {i.attrib["id"]: i.attrib for i in opf.find(f"{pkg}manifest")}
    base = str(Path(rootfile).parent).replace("\\", "/")
    base = "" if base == "." else base + "/"
    missing = [m["href"] for m in manifest.values() if (base + m["href"]) not in z.namelist()]
    check("epub: every manifest item exists", not missing, missing[:3])
    spine = [i.attrib["idref"] for i in opf.find(f"{pkg}spine")]
    check("epub: spine items are in the manifest", all(i in manifest for i in spine), spine[:3])
    nav = [m for m in manifest.values() if "nav" in m.get("properties", "")]
    check("epub: has a navigation document", len(nav) == 1)
    check("epub: has an NCX for older readers", any(m["media-type"] == "application/x-dtbncx+xml" for m in manifest.values()))
    meta = opf.find(f"{pkg}metadata")
    dc = "{http://purl.org/dc/elements/1.1/}"
    check("epub: title, identifier and language", all(meta.find(f"{dc}{t}") is not None for t in ("title", "identifier", "language")))
    check("epub: dcterms:modified present", any(m.attrib.get("property") == "dcterms:modified" for m in meta.findall(f"{pkg}meta")))
    bad = []
    for name in z.namelist():
        if name.endswith((".xhtml", ".html", ".opf", ".ncx", ".xml")):
            try:
                ET.fromstring(z.read(name))
            except ET.ParseError as error:
                bad.append(f"{name}: {error}")
    check("epub: every XML and XHTML part is well-formed", not bad, bad[:2])
    chapters = [m for m in manifest.values() if m["media-type"] == "application/xhtml+xml" and "nav" not in m.get("properties", "")]
    check("epub: one XHTML per document", len(chapters) >= 2, len(chapters))
    return sum(words(re.sub(r"<[^>]+>", " ", z.read(base + c["href"]).decode("utf-8"))) for c in chapters)


def check_fdx(path):
    root = ET.parse(path).getroot()
    check("fdx: root is FinalDraft", root.tag == "FinalDraft", root.tag)
    paragraphs = root.findall(".//Content/Paragraph")
    check("fdx: has paragraphs with types", len(paragraphs) > 2 and all("Type" in p.attrib for p in paragraphs), len(paragraphs))


def main():
    folder = Path(sys.argv[1])
    expect_words = None
    if "--expect-words" in sys.argv:
        expect_words = int(sys.argv[sys.argv.index("--expect-words") + 1])
    tracked = "--tracked" in sys.argv
    counts = {}
    for path in sorted(folder.iterdir()):
        suffix = path.suffix.lower()
        try:
            if suffix == ".md":
                counts["md"] = check_markdown(path)
            elif suffix == ".txt":
                counts["txt"] = check_plain(path)
            elif suffix == ".fountain":
                check_fountain(path)
            elif suffix == ".html":
                check_html(path)
            elif suffix == ".docx":
                counts["docx"] = check_docx(path, tracked)
            elif suffix == ".epub":
                counts["epub"] = check_epub(path)
            elif suffix == ".fdx":
                check_fdx(path)
        except Exception as error:  # report, never hide
            check(f"{path.name}: could be checked", False, repr(error))
    if expect_words:
        for kind, count in counts.items():
            if kind in ("md", "txt", "docx", "epub"):
                # Headings and markers add words; the manuscript's own words must all be there.
                check(f"{kind}: carries the whole manuscript", count >= expect_words * 0.97, f"{count} of about {expect_words} words")
    failed = [r for r in results if not r["ok"]]
    print(json.dumps({"checks": len(results), "failed": len(failed), "results": results, "words": counts}, indent=1))
    sys.exit(1 if failed else 0)


main()
