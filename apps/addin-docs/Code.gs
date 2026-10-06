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
      const text = element.getElement().editAsText();
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
