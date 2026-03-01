var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// _worker.js/index.js
import { r as renderers } from "./chunks/_@astro-renderers_B2-kUL1M.mjs";
import { c as createExports, s as serverEntrypointModule } from "./chunks/_@astrojs-ssr-adapter_jipRO4xr.mjs";
import { manifest } from "./manifest_CmuNKThA.mjs";
globalThis.process ??= {};
globalThis.process.env ??= {};
var serverIslandMap = /* @__PURE__ */ new Map();
var _page0 = /* @__PURE__ */ __name(() => import("./pages/_image.astro.mjs"), "_page0");
var _page1 = /* @__PURE__ */ __name(() => import("./pages/ai.astro.mjs"), "_page1");
var _page2 = /* @__PURE__ */ __name(() => import("./pages/api/_---path_.astro.mjs"), "_page2");
var _page3 = /* @__PURE__ */ __name(() => import("./pages/search.astro.mjs"), "_page3");
var _page4 = /* @__PURE__ */ __name(() => import("./pages/status.astro.mjs"), "_page4");
var _page5 = /* @__PURE__ */ __name(() => import("./pages/index.astro.mjs"), "_page5");
var pageMap = /* @__PURE__ */ new Map([
  ["node_modules/.pnpm/astro@5.18.0_jiti@2.6.1_rollup@4.59.0_typescript@5.9.3_yaml@2.8.2/node_modules/astro/dist/assets/endpoint/generic.js", _page0],
  ["src/pages/ai.astro", _page1],
  ["src/pages/api/[...path].ts", _page2],
  ["src/pages/search.astro", _page3],
  ["src/pages/status.astro", _page4],
  ["src/pages/index.astro", _page5]
]);
var _manifest = Object.assign(manifest, {
  pageMap,
  serverIslandMap,
  renderers,
  actions: /* @__PURE__ */ __name(() => import("./noop-entrypoint.mjs"), "actions"),
  middleware: /* @__PURE__ */ __name(() => import("./_astro-internal_middleware.mjs"), "middleware")
});
var _args = void 0;
var _exports = createExports(_manifest);
var __astrojsSsrVirtualEntry = _exports.default;
var _start = "start";
if (Object.prototype.hasOwnProperty.call(serverEntrypointModule, _start)) {
  serverEntrypointModule[_start](_manifest, _args);
}
export {
  __astrojsSsrVirtualEntry as default,
  pageMap
};
//# sourceMappingURL=bundledWorker-0.9067371540035203.mjs.map
