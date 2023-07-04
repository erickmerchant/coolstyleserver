# coolstyleserver

Cool Style Server is a way to have hot module replacing for CSS when you are not using something like Parcel or Vite. Like say you were working on site with a Java server and you were using Sass to author CSS. It would be nice to have replacing for styles, and it would be a shame to pull in a heavy build tool that's primarily focused on building SPAs. I wrote this initially for my blog which is an Axum app, where I use JS sparingly, but then I realized that it's more broadly useful.

The way Cool Style Server works is you give it the URL to your dev server using the `--proxy` flag and tell it where your CSS is with the `--watch` flag. When an HTML document is served Cool rewrites all links with a rel attribute of stylesheet to be cool-stylesheet custom elements which extend HTMLLinkElement (see [Customized built-in elements](https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_custom_elements#customized_built-in_elements) for more details). Cool-stylesheets are enhanced stylesheets which replace CSS when changes are detected. Replacing happens without a page refresh. Support wise this will only work in browsers that support customized built-in elements, and Constructable Stylesheets. **Also unfortunately stylesheets that contain @import are not supported.**

```
# install with
deno install --allow-read --allow-net https://deno.land/x/coolstyleserver/coolstyleserver.js

# then
coolstyleserver --help
```
