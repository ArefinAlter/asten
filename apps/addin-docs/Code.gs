function onOpen() {
  DocumentApp.getUi()
    .createMenu("Asten")
    .addItem("Open the guest pane", "showSidebar")
    .addToUi();
}

function showSidebar() {
  const html = HtmlService.createHtmlOutputFromFile("Sidebar")
    .setTitle("Asten")
    .setSandboxMode(HtmlService.SandboxMode.IFRAME);
  DocumentApp.getUi().showSidebar(html);
}

function selectedText() {
  const selection = DocumentApp.getActiveDocument().getSelection();
  if (!selection) return "";
  return selection
    .getRangeElements()
    .map((element) => {
      const node = element.getElement();
      // Images, rules and other non-text elements inside a selection have no
      // text to file; skip them rather than throw.
      if (typeof node.editAsText !== "function") return "";
      const text = node.editAsText();
      if (element.isPartial()) {
        return text.getText().substring(
          element.getStartOffset(),
          element.getEndOffsetInclusive() + 1,
        );
      }
      return text.getText();
    })
    .join("\n");
}

// The pairing token lives in the user's own script properties, which are
// private to this Google account and this script. The sidebar's iframe origin
// changes between loads, so browser storage would forget it.
const PAIRING_KEY = "asten.pairing";

function loadPairing() {
  const raw = PropertiesService.getUserProperties().getProperty(PAIRING_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch (error) {
    PropertiesService.getUserProperties().deleteProperty(PAIRING_KEY);
    return null;
  }
}

function savePairing(pairing) {
  PropertiesService.getUserProperties().setProperty(
    PAIRING_KEY,
    JSON.stringify({
      origin: String(pairing.origin),
      token: String(pairing.token),
    }),
  );
  return true;
}

function forgetPairing() {
  PropertiesService.getUserProperties().deleteProperty(PAIRING_KEY);
  return true;
}
