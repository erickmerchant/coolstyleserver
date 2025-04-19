import "./handcraft/dom/css.js";
import "./handcraft/dom/observe.js";
import "./handcraft/dom/on.js";
import {$} from "./handcraft/dom.js";
import {define} from "./handcraft/define.js";
import {watch} from "./handcraft/reactivity.js";

let base = new URL(import.meta.url);
let coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
let esrc = new EventSource(`${coolBase}/watch`);

define("cool-stylesheet")
	.extends("link")
	.connected((host) => {
		let state = watch({css: ""});
		let observed = host.observe();
		let root = host.root();
		let sources = new Set();
		let pathname;
		let url = new URL(host.deref().href);
		let update = async () => {
			let url = new URL(`${coolBase}/fetch`, base);

			url.searchParams.append("pathname", pathname);

			let res = await fetch(url);
			let json = await res.json();

			if (json.css.includes("@import")) {
				return;
			}

			sources = new Set(json.sources);

			state.css = json.css;
		};

		if (url.host !== new URL(import.meta.url).host) return;

		pathname = url.pathname;

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

		root.css(() => state.css, {media: () => observed.attr("media")});
	});
