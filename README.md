# Cool Style Server

Cool Style Server is a development tool, a CLI program, to reload stylesheets in the browser when the underlying source files change without doing a full page refresh, and without using a JavaScript bundler. It is essentially "hot module replacement" for CSS.

## Why

Say you were working on a site with a PHP server and you were using Sass to author CSS. It would be nice to have hot module replacement for styles, and it would be a shame to pull in a heavy build tool that's primarily focused on building SPAs. With Cool Style Server you can just work on your design without pulling in NPM dependencies unnecessarily. It's the ultimate DX for designing in browser. And it is an ally for a more traditional stack. A small piece in a way forward out of JS fatigue. I wrote this initially for my blog which is an Axum app, where I use JS sparingly, but then I realized that it's more broadly useful.

## How

The way Cool Style Server works is you tell it where your CSS is with the first argument. When an HTML document is served C.S.S. rewrites all links with a rel attribute of stylesheet to be cool-stylesheet custom elements which extend HTMLLinkElement (see [Customized built-in elements](https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_custom_elements#customized_built-in_elements) for more details). Rewriting is blazingly fast thanks to [lol-html](https://crates.io/crates/lol-html). `cool-stylesheets` are enhanced stylesheets which replace CSS when changes are detected. Replacing happens without a page refresh. Support wise this will only work in browsers that support customized built-in elements — no safari unfortunately — and Constructable Stylesheets. **Also unfortunately stylesheets that contain @import are not supported.**

## More details

There are two sub-commands: `serve` and `proxy`. `serve` is a full static web server if you don't have a server. Please don't use in production, but it is good for working on simple projects and to prototype an idea. `proxy` is for when you have a server already like in the PHP example above. You pass it the host for example like `coolstyleserver proxy ./static/ http://0.0.0.0:3000`.

Why use custom elements at all? Basically so that this will work for CSS linked to in other custom elements. If you add stylesheet links dynamically you'll need to ensure that they have an attribute of `is` set to `cool-stylesheet`. You can check to see if you should do this by using `customElements.get("cool-stylesheet")`. If you don't use link elements at all C.S.S. can't help you. Server generated CSS files should be supported, but you will need to have source maps in place. I want to support embedded styles soon.

## Installation and Usage

```
# install with
cargo install coolstyleserver

# then
coolstyleserver --help
```
