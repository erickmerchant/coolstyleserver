class CoolStylesheet extends HTMLLinkElement {
  static #nodes = new Map();

  static {
    let base = new URL(import.meta.url);

    base = base.pathname.substring(0, base.pathname.length - 19);

    let esrc = new EventSource(`${base}/watch`);

    esrc.onmessage = (event) => {
      let data = JSON.parse(event.data);

      for (let href of data.hrefs) {
        let nodes = this.#nodes.get(href) ?? [];

        for (let node of nodes) {
          node?.deref()?.updateStyles(href);
        }
      }
    };
  }

  static get observedAttributes() {
    return ["media"];
  }

  #sheet;

  attributeChangedCallback(_, old, value) {
    if (old) this.#sheet?.media?.deleteMedium(old);

    if (value) this.#sheet?.media?.appendMedium(value);
  }

  constructor() {
    super();

    let href = this.getAttribute("href");
    let map = CoolStylesheet.#nodes.get(href) ?? [];
    let root = this.getRootNode();
    let options = {};

    map.push(new WeakRef(this));

    CoolStylesheet.#nodes.set(href, map);

    if (this.hasAttribute("media")) {
      options.media = this.getAttribute("media");
    }

    this.#sheet = this.#sheet ?? new CSSStyleSheet(options);

    this.#sheet.replaceSync("");

    root.adoptedStyleSheets = [...root.adoptedStyleSheets, this.#sheet];
  }

  updateStyles(href) {
    fetch(href)
      .then((res) => res.text())
      .then((css) => {
        if (css.includes("@import")) return;

        this.#sheet.replaceSync(css);

        this.sheet.disabled = true;
      });
  }
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
