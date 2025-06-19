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

define("cool-stylesheet")
	.extends("link")
	.connected((el) => {
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

		$(esrc).on("message", (event) => {
			let data = JSON.parse(event.data);

			for (let p of data) {
				p = new URL(p, base).pathname;

				if (p === pathname || sources.has(p)) {
					update();

					break;
				}
			}
		});

		el.prop("disabled", () => state.updated);

		el.root().css(() => state.css, {media: () => el.attr("media")});

		update();
	});
