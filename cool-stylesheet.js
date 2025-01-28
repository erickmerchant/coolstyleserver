let registry = new Map();
let sources = new Map();
let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

esrc.addEventListener("message", async (event) => {
	let data = JSON.parse(event.data);
	let updates = new Set();

	for (let pathname of data) {
		pathname = new URL(pathname, base).pathname;

		let sheets = sources.get(pathname);

		if (!sheets) continue;

		for (let sheet of sheets) {
			updates.add(updateSheet(sheet, pathname));
		}
	}

	await Promise.allSettled([...updates]);
});

async function createSheet(root, index, pathname, media) {
	media ??= "all";

	let item = registry.get(pathname) ?? new Map();

	registry.set(pathname, item);

	let sheet = item.get(media) ?? new CSSStyleSheet({media: media});

	item.set(media, sheet);

	root.adoptedStyleSheets.splice(index, 1, sheet);

	await updateSheet(sheet, pathname);
}

async function updateSheet(sheet, pathname) {
	let url = new URL(`${coolBase}/fetch`, base);

	url.searchParams.append("pathname", pathname);

	let res = await fetch(url);
	let json = await res.json();

	if (json.css.includes("@import")) {
		return;
	}

	for (let src of [pathname].concat(json.sources ?? [])) {
		sources.set(src, sources.get(src) ?? new Set());

		sources.get(src).add(sheet);
	}

	sheet.replaceSync(json.css);
}

class CoolStylesheet extends HTMLLinkElement {
	static get observedAttributes() {
		return ["media"];
	}

	pathname;

	constructor() {
		super();

		let url = new URL(this.href);

		if (url.host !== new URL(import.meta.url).host) return;

		this.pathname = url.pathname;

		let root = this.getRootNode();
		let media = this.getAttribute("media");

		createSheet(
			root,
			root.adoptedStyleSheets.length,
			this.pathname,
			media
		).then(() => {
			this.disabled = true;
		});
	}

	async attributeChangedCallback(_, old_media, new_media) {
		if (old_media === new_media) {
			return;
		}

		let root = this.getRootNode();
		let old_sheet = registry.get(this.pathname)?.get(old_media ?? "all");
		let index = root.adoptedStyleSheets.lastIndexOf(old_sheet);

		await createSheet(root, index, this.pathname, new_media);
	}
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
