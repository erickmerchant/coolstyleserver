class CoolStylesheet extends HTMLLinkElement {
	static #nodes = new Map();

	static {
		let base = new URL(import.meta.url);

		base = base.pathname.substring(0, base.pathname.lastIndexOf("/"));

		let esrc = new EventSource(`${base}/watch`);

		esrc.addEventListener("message", async (event) => {
			let data = JSON.parse(event.data);
			let updates = [];

			for (let href of data.hrefs) {
				let nodes = this.#nodes.get(href) ?? [];

				for (let node of nodes) {
					updates.push(node?.deref()?.updateStyles(href));
				}
			}

			await Promise.allSettled(updates);
		});
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

	async updateStyles(href) {
		let res = await fetch(href);
		let css = await res.text();

		if (css.includes("@import")) return;

		this.#sheet.replaceSync(css);

		this.sheet.disabled = true;
	}
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
