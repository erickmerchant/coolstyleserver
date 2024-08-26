let registry = new Map();
let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

esrc.addEventListener("message", async (event) => {
  let data = JSON.parse(event.data);
  let updates = [];

  for (let href of data.hrefs) {
    href = new URL(href, base).href;

    let href_sheets = registry.get(href);

    if (!href_sheets) continue;

    for (let sheet of href_sheets.values()) {
      updates.push(updateSheet(sheet, href));
    }
  }

  await Promise.allSettled(updates);
});

async function createSheet(root, href, media) {
  let sheet = new CSSStyleSheet({media: media ?? "all"});
  let href_sheets = registry.get(href) ?? new Map();

  registry.set(href, href_sheets);

  href_sheets.set(media ?? "all", sheet);

  await updateSheet(sheet, href);

  root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
}

async function updateSheet(sheet, href) {
  let res = await fetch(href);
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

  constructor() {
    super();

    let root = this.getRootNode();
    let media = this.getAttribute("media");

    createSheet(root, this.href, media).then(() => {
      this.sheet.disabled = true;
    });
  }

  async attributeChangedCallback(_, old_media, new_media) {
    if (old_media === new_media) {
      return;
    }

    let root = this.getRootNode();

    await createSheet(root, this.href, new_media);

    let href_sheets = registry.get(this.href) ?? new Map();
    let old_sheet = href_sheets.get(old_media ?? "all");

    root.adoptedStyleSheets = [...root.adoptedStyleSheets].filter(
      (sheet) => sheet !== old_sheet
    );
  }
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
