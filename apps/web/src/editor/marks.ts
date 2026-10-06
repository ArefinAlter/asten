import { Mark, Node } from "@tiptap/core";

export const Insertion = Mark.create({
  name: "insertion",
  parseHTML() {
    return [{ tag: "ins" }];
  },
  renderHTML() {
    return ["ins", { class: "tracked-ins" }, 0];
  },
});

export const Deletion = Mark.create({
  name: "deletion",
  parseHTML() {
    return [{ tag: "del" }];
  },
  renderHTML() {
    return ["del", { class: "tracked-del" }, 0];
  },
});

export const SceneBreak = Node.create({
  name: "sceneBreak",
  group: "block",
  parseHTML() {
    return [{ tag: "hr[data-scene]" }];
  },
  renderHTML() {
    return ["hr", { "data-scene": "true", class: "scene-break" }];
  },
});
