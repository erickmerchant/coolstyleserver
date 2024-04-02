# Cool Style Server

Cool Style Server (or C.S.S.) is a way to have hot module replacing for CSS when you are not using something like Parcel or Vite. Like say you were working on a site with a Java server and you were using Sass to author CSS. It would be nice to have hot module replacing for styles, and it would be a shame to pull in a heavy build tool that's primarily focused on building SPAs. I wrote this initially for my blog which is an Axum app, where I use JS sparingly, but then I realized that it's more broadly useful.

The way Cool Style Server works is you tell it where your CSS is with the `--watch` flag, and give it the URL to your dev server using the `--proxy` flag. When an HTML document is served C.S.S. rewrites all links with a rel attribute of stylesheet to be cool-stylesheet custom elements which extend HTMLLinkElement (see [Customized built-in elements](https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_custom_elements#customized_built-in_elements) for more details). `cool-stylesheets` are enhanced stylesheets which replace CSS when changes are detected. Replacing happens without a page refresh. Support wise this will only work in browsers that support customized built-in elements, and Constructable Stylesheets. **Also unfortunately stylesheets that contain @import are not supported.**

Why use custom elements at all? Basically so that this will work for css linked to in other custom elements. If you add stylesheet links dynamically you'll need to ensure that they have an attribute of `is` set to `cool-stylesheet`. You can check to see if you should do this by using `customElements.get("cool-stylesheet")`

## Installation and Usage

```
# install with
cargo install coolstyleserver

# then
coolstyleserver --help
```
