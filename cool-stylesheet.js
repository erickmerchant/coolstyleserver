class CoolStylesheet extends HTMLLinkElement {
	static #sheets = new Map();

	static {
		let base = new URL(import.meta.url);

		let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));

		let esrc = new EventSource(`${coolBase}/watch`);

		esrc.addEventListener("message", async (event) => {
			let data = JSON.parse(event.data);
			let updates = [];

			for (let href of data.hrefs) {
				href = new URL(href, base).href;

				let href_sheets = this.#sheets.get(href);

				if (!href_sheets) continue;

				for (let sheet of href_sheets.values()) {
					updates.push(this.#updateSheet(sheet, href));
				}
			}

			await Promise.allSettled(updates);
		});
	}

	static async #updateSheet(sheet, href) {
		let res = await fetch(href);
		let css = await res.text();

		if (css.includes("@import")) return;

		sheet.replaceSync(css);
	}

	static async #getSheet(href, media) {
		let href_sheets = this.#sheets.get(href) ?? new Map();

		this.#sheets.set(href, href_sheets);

		let sheet = href_sheets.get(media);

		if (!sheet) {
			sheet = new CSSStyleSheet({media});

			href_sheets.set(media, sheet);

			await this.#updateSheet(sheet, href);
		}

		return sheet;
	}

	static get observedAttributes() {
		return ["media"];
	}

	constructor() {
		super();

		this.init();
	}

	attributeChangedCallback(_, old_media, new_media) {
		if (old_media === new_media) return;

		this.init().then(async () => {
			let old_sheet = await CoolStylesheet.#getSheet(
				this.href,
				old_media ?? "screen"
			);
			let root = this.getRootNode();

			root.adoptedStyleSheets = [...root.adoptedStyleSheets].filter(
				(sheet) => sheet !== old_sheet
			);
		});
	}

	async init() {
		let root = this.getRootNode();
		let href = this.href;
		let media = this.getAttribute("media") ?? "screen";
		let sheet = await CoolStylesheet.#getSheet(href, media);

		this.sheet.disabled = true;

		root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
	}
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
