globalThis.process ??= {}; globalThis.process.env ??= {};
/* empty css                              */
import { e as createComponent, k as renderComponent, l as renderScript, r as renderTemplate, m as maybeRenderHead } from '../chunks/astro/server_BTqzngb8.mjs';
import { L as Layout } from '../chunks/Layout_BGy5vrpZ.mjs';
export { r as renderers } from '../chunks/_@astro-renderers_B2-kUL1M.mjs';

const $$Search = createComponent(async ($$result, $$props, $$slots) => {
  return renderTemplate`${renderComponent($$result, "Layout", Layout, { "title": "Domain Search" }, { "default": async ($$result2) => renderTemplate` ${maybeRenderHead()}<div class="space-y-12 max-w-4xl mx-auto"> <div class="space-y-4 text-center"> <h2 class="text-5xl font-black font-outfit tracking-tighter">
Domain <span class="text-gradient">Discovery</span> </h2> <p class="text-gray-500 font-medium max-w-xl mx-auto">
Search the global infrastructure to identify rule classifications for
        any domain.
</p> </div> ${renderComponent($$result2, "search-component", "search-component", { "class": "space-y-12" }, { "default": () => renderTemplate` <div class="glass-panel !p-2 !rounded-full flex items-center bg-white/5 border-white/5 group hover:border-brand-primary/30 transition-all duration-500 max-w-3xl mx-auto"> <div class="pl-6 pr-3 text-gray-500 group-focus-within:text-brand-primary transition-colors"> <div class="i-ph-magnifying-glass-bold text-xl"></div> </div> <input id="search-input" type="text" placeholder="e.g. apple.com" class="flex-1 bg-transparent border-none py-4 text-lg font-medium placeholder:text-gray-600 focus:outline-none text-white tracking-tight"> <button id="search-btn" class="btn-brand !rounded-full py-3 px-8 mx-1">
Search
</button> </div> <div id="results" class="pt-8 min-h-[400px]"> <div class="flex flex-col items-center justify-center text-gray-600 gap-4 opacity-50"> <div class="i-ph-file-search-bold text-6xl"></div> <p class="font-outfit font-bold uppercase tracking-[0.2em] text-xs">
Ready for input
</p> </div> </div> ` })} </div> ` })} ${renderScript($$result, "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/search.astro?astro&type=script&index=0&lang.ts")}`;
}, "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/search.astro", void 0);

const $$file = "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/search.astro";
const $$url = "/search";

const _page = /*#__PURE__*/Object.freeze(/*#__PURE__*/Object.defineProperty({
  __proto__: null,
  default: $$Search,
  file: $$file,
  url: $$url
}, Symbol.toStringTag, { value: 'Module' }));

const page = () => _page;

export { page };
