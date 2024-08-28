let registry = new Map();
let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

esrc.addEventListener("message", async (event) => {
  let data = JSON.parse(event.data);
  let updates = [];

  for (let pathname of data) {
    pathname = new URL(pathname, base).pathname;

    let sheets = registry.get(pathname);

    if (!sheets) continue;

    for (let sheet of sheets.values()) {
      updates.push(updateSheet(sheet, pathname));
    }
  }

  await Promise.allSettled(updates);
});

async function createSheet(root, pathname, media) {
  media ??= "all";

  let sheets = registry.get(pathname) ?? new Map();

  registry.set(pathname, sheets);

  let sheet = sheets.get(media) ?? new CSSStyleSheet({media: media});

  sheets.set(media, sheet);

  await updateSheet(sheet, pathname);

  root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
}

async function updateSheet(sheet, pathname) {
  let res = await fetch(pathname);
  let css = await res.text();

  if (css.includes("@import")) {
    return;
  }

  sheet.replaceSync(css);
}

class CoolStylesheet extends HTMLLinkElement {
  static get observedAttributes() {
    return ["media"];
  }

  pathname;

  constructor() {
    super();

    this.pathname = new URL(this.href).pathname;

    let root = this.getRootNode();
    let media = this.getAttribute("media");

    createSheet(root, this.pathname, media).then(() => {
      this.sheet.disabled = true;
    });
  }

  async attributeChangedCallback(_, old_media, new_media) {
    if (old_media === new_media) {
      return;
    }

    let root = this.getRootNode();

    await createSheet(root, this.pathname, new_media);

    let sheets = registry.get(this.pathname) ?? new Map();
    let old_sheet = sheets.get(old_media ?? "all");

    root.adoptedStyleSheets = [...root.adoptedStyleSheets].filter(
      (sheet) => sheet !== old_sheet
    );
  }
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
