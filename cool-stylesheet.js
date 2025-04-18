let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

class CoolStylesheet extends HTMLLinkElement {
	static get observedAttributes() {
		return ["media"];
	}

	sources = new Set();
	pathname;
	stylesheet;

	async update() {
		let url = new URL(`${coolBase}/fetch`, base);

		url.searchParams.append("pathname", this.pathname);

		let res = await fetch(url);
		let json = await res.json();

		if (json.css.includes("@import")) {
			return;
		}

		this.sources = new Set(json.sources);

		this.stylesheet.replaceSync(json.css);
	}

	connectedCallback() {
		let url = new URL(this.href);

		if (url.host !== new URL(import.meta.url).host) return;

		this.pathname = url.pathname;

		esrc.addEventListener("message", (event) => {
			let data = JSON.parse(event.data);
			let doUpdate = false;

			for (let pathname of data) {
				pathname = new URL(pathname, base).pathname;

				if (pathname !== this.pathname && !this.sources.has(pathname)) continue;

				doUpdate = true;
			}

			if (doUpdate) {
				this.update();
			}
		});

		let media = this.getAttribute("media") ?? "all";

		this.stylesheet = new CSSStyleSheet({media: media});

		let root = this.getRootNode();

		root.adoptedStyleSheets.splice(
			root.adoptedStyleSheets.length,
			1,
			this.stylesheet
		);

		this.update().then(() => {
			this.disabled = true;
		});
	}

	attributeChangedCallback(_, old_media, new_media) {
		if (old_media === new_media) {
			return;
		}

		this.stylesheet.media = new_media;
	}

	disconnectedCallback() {
		let root = this.getRootNode();
		let index = root.adoptedStyleSheets.lastIndexOf(this.stylesheet);

		root.adoptedStyleSheets.splice(index, 1);
	}
}

customElements.define("cool-stylesheet", CoolStylesheet, {extends: "link"});
