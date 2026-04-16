import { $, define, watch } from "@handcraft/lib";

const base = new URL(import.meta.url);
const coolBase = base.pathname.substring(0, base.pathname.lastIndexOf("/"));
const esrc = new EventSource(`${coolBase}/watch`);

$(esrc)
  .on("message", (e) => {
    const data = JSON.parse(e.data);

    for (const { contentType } of data) {
      if (contentType === "text/css") continue;

      globalThis.location.reload();

      e.stopPropagation();

      break;
    }
  });

define("cool-stylesheet", {
  extend: "style",
  href: "",
  connected(el) {
    const url = new URL(this.href, globalThis.location.href);

    if (url.host !== base.host) return;

    const pathname = url.pathname;
    const state = watch({ css: "" });
    let sources = new Set();
    const fetchUrl = new URL(`${coolBase}/fetch${pathname}`, base);

    const update = async () => {
      const res = await fetch(fetchUrl);
      const json = await res.json();

      sources = new Set(
        json.sources.map((s) =>
          (new URL(s, globalThis.location.href)).pathname
        ),
      );

      state.css = json.css;
    };

    $(esrc).on("message", (e) => {
      const data = JSON.parse(e.data);

      for (let { href } of data) {
        href = new URL(href, base).pathname;

        if (href === pathname || sources.has(href)) {
          update();

          break;
        }
      }
    });

    $(el)(() => state.css);

    update();
  },
});
