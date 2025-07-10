import "handcraft/dom/css.js";
import "handcraft/dom/observer.js";
import "handcraft/dom/attr.js";
import "handcraft/dom/on.js";
import "handcraft/dom/prop.js";
import {$} from "handcraft/dom.js";
import {define} from "handcraft/define.js";
import {watch} from "handcraft/reactivity.js";

let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

$(esrc).on("message", (e) => {
	let data = JSON.parse(e.data);

	for (let {contentType} of data) {
		if (contentType === "text/css") continue;

		window.location.reload();

		e.stopPropagation();

		break;
	}
});

define("cool-stylesheet")
	.extends("link")
	.setup((el) => {
		let url = new URL(el.deref().href);

		if (url.host !== base.host) return;

		let pathname = url.pathname;
		let state = watch({updated: false, css: ""});
		let sources = new Set();
		let fetchUrl = new URL(`${coolBase}/fetch${pathname}`, base);

		let update = async () => {
			let res = await fetch(fetchUrl);
			let json = await res.json();

			if (json.css.includes("@import")) {
				return;
			}

			sources = new Set(json.sources);

			state.css = json.css;

			state.updated = true;
		};

		$(esrc).on("message", (e) => {
			let data = JSON.parse(e.data);

			console.log(data);

			for (let {href} of data) {
				href = new URL(href, base).pathname;

				if (href === pathname || sources.has(href)) {
					update();

					break;
				}
			}
		});

		el.prop("disabled", () => state.updated);

		el.root().css(() => state.css, {media: () => el.attr("media")});

		update();
	});
