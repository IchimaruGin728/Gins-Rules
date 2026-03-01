globalThis.process ??= {}; globalThis.process.env ??= {};
/* empty css                              */
import { e as createComponent, k as renderComponent, l as renderScript, r as renderTemplate, h as createAstro, m as maybeRenderHead } from '../chunks/astro/server_BTqzngb8.mjs';
import { L as Layout } from '../chunks/Layout_BGy5vrpZ.mjs';
export { r as renderers } from '../chunks/_@astro-renderers_B2-kUL1M.mjs';

const $$Astro = createAstro();
const $$Status = createComponent(async ($$result, $$props, $$slots) => {
  const Astro2 = $$result.createAstro($$Astro, $$props, $$slots);
  Astro2.self = $$Status;
  return renderTemplate`${renderComponent($$result, "Layout", Layout, { "title": "Status" }, { "default": async ($$result2) => renderTemplate` ${maybeRenderHead()}<div class="space-y-12 max-w-6xl mx-auto"> <div class="space-y-3"> <div class="flex items-center gap-2"> <div class="i-ph-pulse-fill text-2xl text-gradient"></div> <span class="text-[10px] font-black uppercase tracking-[0.3em] text-gray-500">Live Infrastructure</span> </div> <h2 class="text-5xl font-black font-outfit tracking-tighter">
System <span class="text-gradient underline decoration-white/5 underline-offset-4">Status</span> </h2> <p class="text-gray-500 font-medium max-w-xl">
Real-time health, usage, and performance metrics for all Cloudflare edge
        services.
</p> </div> ${renderComponent($$result2, "status-dashboard", "status-dashboard", {}, { "default": () => renderTemplate` <div id="status-loading" class="flex flex-col items-center justify-center py-20 gap-4"> <div class="w-12 h-12 rounded-full border-2 border-brand-primary/20 border-t-brand-primary animate-spin"></div> <p class="font-outfit font-bold uppercase tracking-[0.2em] text-[10px] text-brand-primary animate-pulse">
Querying Cloudflare Edge...
</p> </div> <div id="status-content" class="hidden space-y-10"> <div class="grid grid-cols-1 md:grid-cols-2 gap-6"> <div class="glass-panel border-white/5 relative overflow-hidden group"> <div class="flex items-center justify-between mb-6"> <div class="flex items-center gap-3"> <div class="w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center group-hover:bg-brand-primary/10 transition-colors"> <div class="i-ph-robot-bold text-xl text-brand-primary"></div> </div> <div> <div class="font-bold font-outfit tracking-tight">
Scanner Worker
</div> <div class="text-[10px] font-black uppercase tracking-widest text-gray-600">
gins-rules-scanner
</div> </div> </div> <div id="scanner-status" class="flex items-center gap-2 px-3 py-1 rounded-full border text-[9px] font-black uppercase"> <div class="w-2 h-2 rounded-full animate-pulse"></div> <span>Checking...</span> </div> </div> <div class="grid grid-cols-2 gap-4"> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
Requests 24h
</div> <div id="scanner-requests" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
Errors 24h
</div> <div id="scanner-errors" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
CPU P50
</div> <div id="scanner-cpu50" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
CPU P99
</div> <div id="scanner-cpu99" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> </div> </div> <div class="glass-panel border-white/5 relative overflow-hidden group"> <div class="flex items-center justify-between mb-6"> <div class="flex items-center gap-3"> <div class="w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center group-hover:bg-brand-primary/10 transition-colors"> <div class="i-ph-brain-bold text-xl text-brand-primary"></div> </div> <div> <div class="font-bold font-outfit tracking-tight">
Workers AI
</div> <div class="text-[10px] font-black uppercase tracking-widest text-gray-600">
Llama-4-Scout-17B
</div> </div> </div> <div class="flex items-center gap-2 px-3 py-1 rounded-full border border-brand-primary/30 bg-brand-primary/10 text-[9px] font-black uppercase text-brand-primary"> <div class="w-2 h-2 rounded-full bg-brand-primary animate-pulse"></div> <span>MoE Active</span> </div> </div> <div class="grid grid-cols-2 gap-4"> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
Inferences 24h
</div> <div id="ai-requests" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> <div class="bg-white/[0.03] rounded-xl p-4 border border-white/5"> <div class="text-[10px] font-black uppercase tracking-widest text-gray-500 mb-1">
Tokens 24h
</div> <div id="ai-tokens" class="text-2xl font-black font-outfit tracking-tighter">
—
</div> </div> </div> </div> </div> <div class="grid grid-cols-1 md:grid-cols-3 gap-6"> <div class="glass-panel border-white/5 group hover:scale-[1.02] duration-500"> <div class="p-4 bg-white/5 rounded-2xl mb-4 w-14 h-14 flex items-center justify-center text-brand-primary group-hover:bg-brand-primary/10 transition-colors"> <div class="i-ph-globe-bold text-3xl"></div> </div> <div id="cdn-requests" class="text-4xl font-black font-outfit tracking-tighter">
—
</div> <div class="text-gray-500 text-sm font-bold uppercase tracking-widest mt-1">
CDN Requests 24h
</div> </div> <div class="glass-panel border-white/5 group hover:scale-[1.02] duration-500"> <div class="p-4 bg-white/5 rounded-2xl mb-4 w-14 h-14 flex items-center justify-center text-brand-primary group-hover:bg-brand-primary/10 transition-colors"> <div class="i-ph-cloud-arrow-up-bold text-3xl"></div> </div> <div id="cdn-bandwidth" class="text-4xl font-black font-outfit tracking-tighter">
—
</div> <div class="text-gray-500 text-sm font-bold uppercase tracking-widest mt-1">
Bandwidth 24h
</div> </div> <div class="glass-panel border-white/5 group hover:scale-[1.02] duration-500"> <div class="p-4 bg-white/5 rounded-2xl mb-4 w-14 h-14 flex items-center justify-center text-brand-primary group-hover:bg-brand-primary/10 transition-colors"> <div class="i-ph-lightning-bold text-3xl"></div> </div> <div id="total-workers" class="text-4xl font-black font-outfit tracking-tighter">
—
</div> <div class="text-gray-500 text-sm font-bold uppercase tracking-widest mt-1">
Total Worker Reqs
</div> </div> </div> <div class="glass-panel border-white/5 !p-0 overflow-hidden"> <div class="bg-white/[0.03] p-6 border-b border-white/5 flex justify-between items-center"> <h4 class="font-bold font-outfit tracking-tight text-xl">
Service Endpoints
</h4> <div id="status-time" class="text-[10px] font-black uppercase tracking-widest text-gray-600"></div> </div> <div class="divide-y divide-white/5"> <a href="https://gins-rules-scanner.ichimarugin728.workers.dev" target="_blank" class="p-5 flex items-center justify-between hover:bg-white/[0.02] transition-colors group"> <div class="flex items-center gap-4"> <div class="i-ph-hard-drives-bold text-xl text-gray-500 group-hover:text-brand-primary transition-colors"></div> <div> <div class="font-bold font-mono text-sm">
gins-rules-scanner.workers.dev
</div> <div class="text-[10px] text-gray-600 font-bold uppercase tracking-widest">
Scanner Worker · Hono API
</div> </div> </div> <div class="i-ph-arrow-square-out-bold text-gray-600 group-hover:text-brand-primary transition-colors"></div> </a> <a href="https://gins-rules.pages.dev" target="_blank" class="p-5 flex items-center justify-between hover:bg-white/[0.02] transition-colors group"> <div class="flex items-center gap-4"> <div class="i-ph-hard-drives-bold text-xl text-gray-500 group-hover:text-brand-primary transition-colors"></div> <div> <div class="font-bold font-mono text-sm">
gins-rules.pages.dev
</div> <div class="text-[10px] text-gray-600 font-bold uppercase tracking-widest">
Rules CDN · Cloudflare Pages
</div> </div> </div> <div class="i-ph-arrow-square-out-bold text-gray-600 group-hover:text-brand-primary transition-colors"></div> </a> <a href="https://gins-rules-dashboard.pages.dev" target="_blank" class="p-5 flex items-center justify-between hover:bg-white/[0.02] transition-colors group"> <div class="flex items-center gap-4"> <div class="i-ph-hard-drives-bold text-xl text-gray-500 group-hover:text-brand-primary transition-colors"></div> <div> <div class="font-bold font-mono text-sm">
gins-rules-dashboard.pages.dev
</div> <div class="text-[10px] text-gray-600 font-bold uppercase tracking-widest">
Dashboard · Astro SSR
</div> </div> </div> <div class="i-ph-arrow-square-out-bold text-gray-600 group-hover:text-brand-primary transition-colors"></div> </a> </div> </div> </div> ` })} </div> ` })} ${renderScript($$result, "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/status.astro?astro&type=script&index=0&lang.ts")}`;
}, "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/status.astro", void 0);

const $$file = "/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/status.astro";
const $$url = "/status";

const _page = /*#__PURE__*/Object.freeze(/*#__PURE__*/Object.defineProperty({
  __proto__: null,
  default: $$Status,
  file: $$file,
  url: $$url
}, Symbol.toStringTag, { value: 'Module' }));

const page = () => _page;

export { page };
