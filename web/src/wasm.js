// Lazy-loader for the wasm-bindgen bundle produced by the Rust build.
//
// The bundle is emitted to `public/pkg/` by `scripts/build-wasm.sh` and is a
// plain ES module (`--target web`) that instantiates the `.wasm` at
// `import` time. We dynamic-import it with a runtime-computed URL so:
//   * Rollup leaves it as-is (no bundler resolution error), and
//   * it works whether the site is served at the origin root (dev) or under
//     a GitHub Pages project sub-path (e.g. /blindroute/).

let instancePromise = null;

/**
 * @returns {Promise<any>} the wasm module (blindroute_wasm.js)
 */
export function loadWasm() {
  if (!instancePromise) {
    let base = document.baseURI;
    if (!base.endsWith('/')) base = new URL('./', base).href;
    const url = new URL('pkg/blindroute_wasm.js?v=021', base).href;
    instancePromise = import(/* @vite-ignore */ url).then(async (mod) => {
      await mod.default();
      return mod;
    }).catch((e) => {
      instancePromise = null;
      throw e;
    });
  }
  return instancePromise;
}
