import {concat} from "https://deno.land/std@0.182.0/bytes/mod.ts";
import {parse} from "https://deno.land/std@0.182.0/flags/mod.ts";
import {serve} from "https://deno.land/std@0.182.0/http/mod.ts";
import {resolve} from "https://deno.land/std@0.182.0/path/mod.ts";
import init, {HTMLRewriter} from "https://deno.land/x/lol_html@0.0.6/mod.ts";

function client() {
  class CoolStylesheet extends HTMLLinkElement {
    static #nodes = new Map();

    static {
      let base = new URL(import.meta.url);

      base = base.pathname.substring(0, base.pathname.length - 10);

      let esrc = new EventSource(`${base}/changes`);

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
}

let usage = `
$ coolstyleserver [options]
-P, --port=<number>     The port to listen at [default: 4000]
-p, --proxy=<url>       Your dev server. Include the protocol. Also the port if it's not 80 [default: http://0.0.0.0:3000]
-w, --watch=<dir>       The directory where your CSS is. [default: ./public]
-b, --base=<dir>        Set if /coolstyle conflicts with a route on your dev server [default: /coolstyle]
-h, --help              Print this message
`.trim();

async function cli() {
  let flags = parse(Deno.args, {
    string: ["proxy", "watch", "base"],
    number: ["port"],
    boolean: ["help"],
    alias: {proxy: "p", port: "P", watch: "w", base: "b", help: "h"},
    default: {
      proxy: "http://0.0.0.0:3000",
      port: 4000,
      watch: "./public",
      base: "/coolstyle",
    },
  });

  if (flags.help) {
    console.log(usage);

    return;
  }

  await init();

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

    if (path === `${flags.base}/changes`) {
      let watcher = Deno.watchFs(flags.watch);
      let hrefs = new Set();
      let enc = new TextEncoder();
      let body = new ReadableStream({
        async start(controller) {
          controller.enqueue(enc.encode(`\n\n`));

          let watchDir = resolve(flags.watch);

          for await (let e of watcher) {
            for (let href of e.paths
              .filter((p) => p.endsWith(".css"))
              .map((p) => p.substring(watchDir.length))) {
              hrefs.add(href);
            }

            setTimeout(() => {
              if (hrefs.size) {
                let data = JSON.stringify({
                  hrefs: [...hrefs.values()],
                });

                hrefs.clear();

                controller.enqueue(enc.encode(`data: ${data}\n\n`));
              }
            }, 10);
          }
        },
        cancel() {
          watcher.close();
        },
      });

      return new Response(body, {
        headers: {
          "content-type": "text/event-stream",
          "cache-control": "no-cache",
        },
      });
    }

    try {
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

      if (contentType?.startsWith("text/html")) {
        body = await proxyRes.text();

        let enc = new TextEncoder();
        let dec = new TextDecoder();
        let chunks = [];
        let rewriter = new HTMLRewriter("utf8", (chunk) => {
          chunks.push(chunk);
        });

        rewriter.on("link[rel=stylesheet]", {
          element(el) {
            el.setAttribute("is", "cool-stylesheet");

            el.after(
              `<script type="module" src="${flags.base}/client.js"></script>`,
              {
                html: true,
              }
            );
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
    } catch (e) {
      let headers = {
        "content-type": "text/html",
      };

      if (e instanceof TypeError) {
        headers.refresh = "1";
      }

      return new Response("", {
        status: 200,
        headers,
      });
    }
  };

  serve(reqHandler, {port: flags.port});
}

if (import.meta.main) {
  cli();
}
