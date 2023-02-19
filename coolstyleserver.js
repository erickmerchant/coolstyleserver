import {serve} from "https://deno.land/std@0.177.0/http/mod.ts";
import {concat} from "https://deno.land/std@0.177.0/bytes/mod.ts";
import init, {HTMLRewriter} from "https://deno.land/x/lol_html@0.0.6/mod.ts";
import wasm from "https://deno.land/x/lol_html@0.0.6/wasm.js";
import {resolve} from "https://deno.land/std@0.177.0/path/mod.ts";
import {parse} from "https://deno.land/std@0.177.0/flags/mod.ts";

function client() {
  class DevStylesheet extends HTMLLinkElement {
    static #nodes = new Map();

    static {
      let base = new URL(import.meta.url);

      base = base.pathname.substring(0, base.pathname.length - 10);

      let esrc = new EventSource(`${base}/_changes`);

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

      let map = DevStylesheet.#nodes.get(href) ?? [];

      map.push(new WeakRef(this));

      DevStylesheet.#nodes.set(href, map);

      let options = {};

      if (this.hasAttribute("media")) {
        options.media = this.getAttribute("media");
      }

      this.#sheet = this.#sheet ?? new CSSStyleSheet(options);

      this.#sheet.replaceSync("");

      let root = this.getRootNode();

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

  customElements.define("dev-stylesheet", DevStylesheet, {extends: "link"});
}

let usage = `
$ coolstyleserver [options]
-p, --proxy=<url>       Your dev server. Include the protocol. Also the port if it's not 80 [default: http://0.0.0.0:3000]
-w, --watch=<dir>       The directory where your CSS is. [default: ./public]
-b, --base=<dir>        Set if /dev conflicts with a route on your dev server [default: /dev]
-h, --help              Print this message
`.trim();

async function cli() {
  let flags = parse(Deno.args, {
    string: ["proxy", "watch", "base"],
    boolean: ["help"],
    alias: {proxy: "p", watch: "w", base: "b", help: "h"},
    default: {proxy: "http://0.0.0.0:3000", watch: "./public", base: "/dev"},
  });

  if (flags.help) {
    console.log(usage);

    return;
  }

  await init(wasm());

  let reqHandler = async (req) => {
    let path = new URL(req.url).pathname;

    if (path === `${flags.base}/client.js`) {
      let body = `${client.toString()}; client()`;

      return new Response(body, {
        headers: {
          "content-type": "application/javascript",
        },
      });
    }

    if (path === `${flags.base}/_changes`) {
      let watcher = Deno.watchFs(flags.watch);
      let enc = new TextEncoder();

      let body = new ReadableStream({
        async start(controller) {
          controller.enqueue(enc.encode(`\n\n`));

          let watchDir = resolve(flags.watch);

          for await (let e of watcher) {
            let data = JSON.stringify({
              hrefs: e.paths
                .filter((p) => p.endsWith(".css"))
                .map((p) => p.substring(watchDir.length)),
            });
            controller.enqueue(enc.encode(`data: ${data}\n\n`));
          }
        },
        cancel() {
          watcher.close();
        },
      });

      return new Response(body, {
        headers: {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
        },
      });
    }

    let proxyRes = await fetch(flags.proxy + path);
    let headers = new Headers();
    let contentType;
    let body = proxyRes.body;

    for (let [name, value] of proxyRes.headers) {
      if (name === "content-type") {
        contentType = value;
      }

      headers.set(name, value);
    }

    if (contentType.startsWith("text/html")) {
      body = await proxyRes.text();

      let enc = new TextEncoder();
      let dec = new TextDecoder();
      let chunks = [];

      let rewriter = new HTMLRewriter("utf8", (chunk) => {
        chunks.push(chunk);
      });

      rewriter.on("link[rel=stylesheet]", {
        element(el) {
          el.setAttribute("is", "dev-stylesheet");

          el.after(
            `<script type="module" src="${flags.base}/client.js"></script>`,
            {
              html: true,
            }
          );
        },
      });

      rewriter.on("link[rel=preload][as=style]", {
        element(el) {
          el.remove();
        },
      });

      rewriter.write(enc.encode(body));

      try {
        rewriter.end();
        body = dec.decode(concat(...chunks));
      } finally {
        rewriter.free();
      }
    }

    return new Response(body, {
      status: proxyRes.status,
      headers,
    });
  };

  serve(reqHandler, {port: 4000});
}

if (import.meta.main) {
  cli();
}
