globalThis.process ??= {}; globalThis.process.env ??= {};
import { n as decodeKey } from './chunks/astro/server_BTqzngb8.mjs';
import './chunks/astro-designed-error-pages_DTrXhg6j.mjs';
import { N as NOOP_MIDDLEWARE_FN } from './chunks/noop-middleware_CMcsHAPi.mjs';

function sanitizeParams(params) {
  return Object.fromEntries(
    Object.entries(params).map(([key, value]) => {
      if (typeof value === "string") {
        return [key, value.normalize().replace(/#/g, "%23").replace(/\?/g, "%3F")];
      }
      return [key, value];
    })
  );
}
function getParameter(part, params) {
  if (part.spread) {
    return params[part.content.slice(3)] || "";
  }
  if (part.dynamic) {
    if (!params[part.content]) {
      throw new TypeError(`Missing parameter: ${part.content}`);
    }
    return params[part.content];
  }
  return part.content.normalize().replace(/\?/g, "%3F").replace(/#/g, "%23").replace(/%5B/g, "[").replace(/%5D/g, "]");
}
function getSegment(segment, params) {
  const segmentPath = segment.map((part) => getParameter(part, params)).join("");
  return segmentPath ? "/" + segmentPath : "";
}
function getRouteGenerator(segments, addTrailingSlash) {
  return (params) => {
    const sanitizedParams = sanitizeParams(params);
    let trailing = "";
    if (addTrailingSlash === "always" && segments.length) {
      trailing = "/";
    }
    const path = segments.map((segment) => getSegment(segment, sanitizedParams)).join("") + trailing;
    return path || "/";
  };
}

function deserializeRouteData(rawRouteData) {
  return {
    route: rawRouteData.route,
    type: rawRouteData.type,
    pattern: new RegExp(rawRouteData.pattern),
    params: rawRouteData.params,
    component: rawRouteData.component,
    generate: getRouteGenerator(rawRouteData.segments, rawRouteData._meta.trailingSlash),
    pathname: rawRouteData.pathname || void 0,
    segments: rawRouteData.segments,
    prerender: rawRouteData.prerender,
    redirect: rawRouteData.redirect,
    redirectRoute: rawRouteData.redirectRoute ? deserializeRouteData(rawRouteData.redirectRoute) : void 0,
    fallbackRoutes: rawRouteData.fallbackRoutes.map((fallback) => {
      return deserializeRouteData(fallback);
    }),
    isIndex: rawRouteData.isIndex,
    origin: rawRouteData.origin
  };
}

function deserializeManifest(serializedManifest) {
  const routes = [];
  for (const serializedRoute of serializedManifest.routes) {
    routes.push({
      ...serializedRoute,
      routeData: deserializeRouteData(serializedRoute.routeData)
    });
    const route = serializedRoute;
    route.routeData = deserializeRouteData(serializedRoute.routeData);
  }
  const assets = new Set(serializedManifest.assets);
  const componentMetadata = new Map(serializedManifest.componentMetadata);
  const inlinedScripts = new Map(serializedManifest.inlinedScripts);
  const clientDirectives = new Map(serializedManifest.clientDirectives);
  const serverIslandNameMap = new Map(serializedManifest.serverIslandNameMap);
  const key = decodeKey(serializedManifest.key);
  return {
    // in case user middleware exists, this no-op middleware will be reassigned (see plugin-ssr.ts)
    middleware() {
      return { onRequest: NOOP_MIDDLEWARE_FN };
    },
    ...serializedManifest,
    assets,
    componentMetadata,
    inlinedScripts,
    clientDirectives,
    routes,
    serverIslandNameMap,
    key
  };
}

const manifest = deserializeManifest({"hrefRoot":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/","cacheDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/node_modules/.astro/","outDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/dist/","srcDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/","publicDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/public/","buildClientDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/dist/","buildServerDir":"file:///Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/dist/_worker.js/","adapterName":"@astrojs/cloudflare","routes":[{"file":"","links":[],"scripts":[],"styles":[],"routeData":{"type":"page","component":"_server-islands.astro","params":["name"],"segments":[[{"content":"_server-islands","dynamic":false,"spread":false}],[{"content":"name","dynamic":true,"spread":false}]],"pattern":"^\\/_server-islands\\/([^/]+?)\\/?$","prerender":false,"isIndex":false,"fallbackRoutes":[],"route":"/_server-islands/[name]","origin":"internal","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[],"routeData":{"type":"endpoint","isIndex":false,"route":"/_image","pattern":"^\\/_image\\/?$","segments":[[{"content":"_image","dynamic":false,"spread":false}]],"params":[],"component":"node_modules/.pnpm/astro@5.18.0_jiti@2.6.1_rollup@4.59.0_typescript@5.9.3_yaml@2.8.2/node_modules/astro/dist/assets/endpoint/generic.js","pathname":"/_image","prerender":false,"fallbackRoutes":[],"origin":"internal","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[{"type":"external","src":"/_astro/ai.BaM8-7v_.css"}],"routeData":{"route":"/ai","isIndex":false,"type":"page","pattern":"^\\/ai\\/?$","segments":[[{"content":"ai","dynamic":false,"spread":false}]],"params":[],"component":"src/pages/ai.astro","pathname":"/ai","prerender":false,"fallbackRoutes":[],"distURL":[],"origin":"project","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[],"routeData":{"route":"/api/[...path]","isIndex":false,"type":"endpoint","pattern":"^\\/api(?:\\/(.*?))?\\/?$","segments":[[{"content":"api","dynamic":false,"spread":false}],[{"content":"...path","dynamic":true,"spread":true}]],"params":["...path"],"component":"src/pages/api/[...path].ts","prerender":false,"fallbackRoutes":[],"distURL":[],"origin":"project","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[{"type":"external","src":"/_astro/ai.BaM8-7v_.css"}],"routeData":{"route":"/search","isIndex":false,"type":"page","pattern":"^\\/search\\/?$","segments":[[{"content":"search","dynamic":false,"spread":false}]],"params":[],"component":"src/pages/search.astro","pathname":"/search","prerender":false,"fallbackRoutes":[],"distURL":[],"origin":"project","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[{"type":"external","src":"/_astro/ai.BaM8-7v_.css"}],"routeData":{"route":"/status","isIndex":false,"type":"page","pattern":"^\\/status\\/?$","segments":[[{"content":"status","dynamic":false,"spread":false}]],"params":[],"component":"src/pages/status.astro","pathname":"/status","prerender":false,"fallbackRoutes":[],"distURL":[],"origin":"project","_meta":{"trailingSlash":"ignore"}}},{"file":"","links":[],"scripts":[],"styles":[{"type":"external","src":"/_astro/ai.BaM8-7v_.css"}],"routeData":{"route":"/","isIndex":true,"type":"page","pattern":"^\\/$","segments":[],"params":[],"component":"src/pages/index.astro","pathname":"/","prerender":false,"fallbackRoutes":[],"distURL":[],"origin":"project","_meta":{"trailingSlash":"ignore"}}}],"base":"/","trailingSlash":"ignore","compressHTML":true,"componentMetadata":[],"renderers":[],"clientDirectives":[["idle","(()=>{var l=(n,t)=>{let i=async()=>{await(await n())()},e=typeof t.value==\"object\"?t.value:void 0,s={timeout:e==null?void 0:e.timeout};\"requestIdleCallback\"in window?window.requestIdleCallback(i,s):setTimeout(i,s.timeout||200)};(self.Astro||(self.Astro={})).idle=l;window.dispatchEvent(new Event(\"astro:idle\"));})();"],["load","(()=>{var e=async t=>{await(await t())()};(self.Astro||(self.Astro={})).load=e;window.dispatchEvent(new Event(\"astro:load\"));})();"],["media","(()=>{var n=(a,t)=>{let i=async()=>{await(await a())()};if(t.value){let e=matchMedia(t.value);e.matches?i():e.addEventListener(\"change\",i,{once:!0})}};(self.Astro||(self.Astro={})).media=n;window.dispatchEvent(new Event(\"astro:media\"));})();"],["only","(()=>{var e=async t=>{await(await t())()};(self.Astro||(self.Astro={})).only=e;window.dispatchEvent(new Event(\"astro:only\"));})();"],["visible","(()=>{var a=(s,i,o)=>{let r=async()=>{await(await s())()},t=typeof i.value==\"object\"?i.value:void 0,c={rootMargin:t==null?void 0:t.rootMargin},n=new IntersectionObserver(e=>{for(let l of e)if(l.isIntersecting){n.disconnect(),r();break}},c);for(let e of o.children)n.observe(e)};(self.Astro||(self.Astro={})).visible=a;window.dispatchEvent(new Event(\"astro:visible\"));})();"]],"entryModules":{"\u0000astro-internal:middleware":"_astro-internal_middleware.mjs","\u0000virtual:astro:actions/noop-entrypoint":"noop-entrypoint.mjs","\u0000@astro-page:src/pages/ai@_@astro":"pages/ai.astro.mjs","\u0000@astro-page:src/pages/api/[...path]@_@ts":"pages/api/_---path_.astro.mjs","\u0000@astro-page:src/pages/search@_@astro":"pages/search.astro.mjs","\u0000@astro-page:src/pages/status@_@astro":"pages/status.astro.mjs","\u0000@astro-page:src/pages/index@_@astro":"pages/index.astro.mjs","\u0000@astrojs-ssr-virtual-entry":"index.js","\u0000@astro-page:node_modules/.pnpm/astro@5.18.0_jiti@2.6.1_rollup@4.59.0_typescript@5.9.3_yaml@2.8.2/node_modules/astro/dist/assets/endpoint/generic@_@js":"pages/_image.astro.mjs","\u0000@astro-renderers":"renderers.mjs","\u0000@astrojs-ssr-adapter":"_@astrojs-ssr-adapter.mjs","\u0000@astrojs-manifest":"manifest_CmuNKThA.mjs","/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/node_modules/.pnpm/unstorage@1.17.4/node_modules/unstorage/drivers/cloudflare-kv-binding.mjs":"chunks/cloudflare-kv-binding_DMly_2Gl.mjs","/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/node_modules/.pnpm/astro@5.18.0_jiti@2.6.1_rollup@4.59.0_typescript@5.9.3_yaml@2.8.2/node_modules/astro/dist/assets/services/noop.js":"chunks/noop_Desx2XLg.mjs","/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/search.astro?astro&type=script&index=0&lang.ts":"_astro/search.astro_astro_type_script_index_0_lang.CFELB50F.js","/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/status.astro?astro&type=script&index=0&lang.ts":"_astro/status.astro_astro_type_script_index_0_lang.BXg7jVVI.js","/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/node_modules/.pnpm/@preact+signals@2.8.1_preact@10.28.4/node_modules/@preact/signals/dist/signals.module.js":"_astro/signals.module.CY599cp-.js","@astrojs/preact/client.js":"_astro/client.Dt-HMUee.js","astro:scripts/before-hydration.js":""},"inlinedScripts":[["/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/search.astro?astro&type=script&index=0&lang.ts","class l extends HTMLElement{constructor(){super();const i=this.querySelector(\"#search-input\"),d=this.querySelector(\"#search-btn\"),s=this.querySelector(\"#results\"),n=async()=>{const a=i.value.trim();if(a){s.innerHTML=`\n          <div class=\"flex flex-col items-center justify-center py-20 gap-4\">\n            <div class=\"w-12 h-12 rounded-full border-2 border-brand-primary/20 border-t-brand-primary animate-spin\" />\n            <p class=\"font-outfit font-bold uppercase tracking-[0.2em] text-[10px] text-brand-primary animate-pulse\">Scanning DB-1 Cloud...</p>\n          </div>\n        `;try{const t=await(await fetch(`/api/search?q=${encodeURIComponent(a)}`)).json();if(t.domains.length===0&&t.rules.length===0){s.innerHTML=`\n              <div class=\"glass-panel text-center py-16 opacity-60\">\n                <div class=\"i-ph-ghost-bold text-4xl mx-auto mb-4 text-gray-500\" />\n                <p class=\"font-outfit font-bold uppercase tracking-widest text-xs italic\">No matching records found.</p>\n              </div>\n            `;return}let e='<div class=\"grid grid-cols-1 lg:grid-cols-2 gap-12\">';t.domains.length>0&&(e+='<div class=\"space-y-6\">',e+='<h4 class=\"text-[10px] font-black uppercase text-gray-500 tracking-[0.3em] pl-2 border-l-2 border-brand-primary\">Direct Matches</h4>',e+='<div class=\"grid gap-4\">'+t.domains.map(r=>`\n              <div class=\"glass-panel !p-5 flex items-center justify-between group hover:scale-[1.02] duration-500 bg-white/[0.03]\">\n                <span class=\"font-mono text-white group-hover:text-brand-primary transition-colors\">${r.domain}</span>\n                <span class=\"text-[10px] bg-white/5 px-3 py-1 rounded-full text-gray-400 font-bold border border-white/5 uppercase tracking-widest\">${r.category||\"PENDING\"}</span>\n              </div>\n            `).join(\"\")+\"</div></div>\"),t.rules.length>0&&(e+='<div class=\"space-y-6\">',e+='<h4 class=\"text-[10px] font-black uppercase text-gray-500 tracking-[0.3em] pl-2 border-l-2 border-brand-primary\">Module Groups</h4>',e+='<div class=\"grid gap-4\">'+t.rules.map(r=>`\n              <div class=\"glass-panel !p-5 flex items-center justify-between hover:scale-[1.02] duration-500 bg-white/[0.03] border-white/10\">\n                <div class=\"flex items-center gap-4\">\n                  <div class=\"w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center text-brand-primary\">\n                    <div class=\"i-ph-folder-simple-user-bold text-xl\" />\n                  </div>\n                  <span class=\"font-bold font-outfit text-lg tracking-tight\">${r.name}</span>\n                </div>\n                <div class=\"text-right\">\n                  <div class=\"text-[10px] font-black text-brand-primary uppercase tracking-widest\">${r.category}</div>\n                  <div class=\"text-[10px] text-gray-500 font-medium\">${r.ruleCount} Entries</div>\n                </div>\n              </div>\n            `).join(\"\")+\"</div></div>\"),e+=\"</div>\",s.innerHTML=e}catch{s.innerHTML='<div class=\"glass-panel !bg-red-500/10 !border-red-500/30 text-center py-10 font-bold text-red-400\">EDGE ENGINE OFFLINE</div>'}}};d?.addEventListener(\"click\",n),i?.addEventListener(\"keypress\",a=>a.key===\"Enter\"&&n())}}customElements.define(\"search-component\",l);"],["/Users/ichimarugin728/Gins-Configs/Gins-Rules/dashboard/src/pages/status.astro?astro&type=script&index=0&lang.ts","class o extends HTMLElement{constructor(){super(),this.load()}async load(){const e=this.querySelector(\"#status-loading\"),r=this.querySelector(\"#status-content\");try{const t=await(await fetch(\"/api/status\")).json(),s=this.querySelector(\"#scanner-status\"),a=s.querySelector(\"div\"),i=s.querySelector(\"span\");t.scanner.healthy?(s.className=\"flex items-center gap-2 px-3 py-1 rounded-full border border-green-500/30 bg-green-500/10 text-[9px] font-black uppercase text-green-400\",a.className=\"w-2 h-2 rounded-full bg-green-500 animate-pulse\",i.textContent=\"Operational\"):(s.className=\"flex items-center gap-2 px-3 py-1 rounded-full border border-red-500/30 bg-red-500/10 text-[9px] font-black uppercase text-red-400\",a.className=\"w-2 h-2 rounded-full bg-red-500 animate-pulse\",i.textContent=\"Degraded\"),this.set(\"#scanner-requests\",this.fmt(t.scanner.requests24h)),this.set(\"#scanner-errors\",this.fmt(t.scanner.errors24h)),this.set(\"#scanner-cpu50\",`${(t.scanner.cpuP50/1e3).toFixed(1)}ms`),this.set(\"#scanner-cpu99\",`${(t.scanner.cpuP99/1e3).toFixed(1)}ms`),this.set(\"#ai-requests\",this.fmt(t.ai.requests24h)),this.set(\"#ai-tokens\",this.fmt(t.ai.tokens24h)),this.set(\"#cdn-requests\",this.fmt(t.cdn.requests24h)),this.set(\"#cdn-bandwidth\",this.bytes(t.cdn.bandwidth24h)),this.set(\"#total-workers\",this.fmt(t.totalWorkerRequests)),this.set(\"#status-time\",`Updated ${new Date(t.timestamp).toLocaleTimeString()}`),e.classList.add(\"hidden\"),r.classList.remove(\"hidden\")}catch{e.innerHTML='<div class=\"glass-panel !bg-red-500/10 !border-red-500/30 text-center py-10 font-bold text-red-400\">Failed to load metrics</div>'}}set(e,r){const n=this.querySelector(e);n&&(n.textContent=r)}fmt(e){return e>=1e6?`${(e/1e6).toFixed(1)}M`:e>=1e3?`${(e/1e3).toFixed(1)}K`:String(e)}bytes(e){return e>=1073741824?`${(e/1073741824).toFixed(1)} GB`:e>=1048576?`${(e/1048576).toFixed(1)} MB`:e>=1024?`${(e/1024).toFixed(1)} KB`:`${e} B`}}customElements.define(\"status-dashboard\",o);"]],"assets":["/_astro/ai.BaM8-7v_.css","/favicon.ico","/favicon.svg","/_astro/client.BTOe5lF4.js","/_astro/client.Dt-HMUee.js","/_astro/signals.module.CY599cp-.js","/_worker.js/_@astrojs-ssr-adapter.mjs","/_worker.js/_astro-internal_middleware.mjs","/_worker.js/index.js","/_worker.js/noop-entrypoint.mjs","/_worker.js/renderers.mjs","/_worker.js/_astro/ai.BaM8-7v_.css","/_worker.js/pages/_image.astro.mjs","/_worker.js/pages/ai.astro.mjs","/_worker.js/pages/index.astro.mjs","/_worker.js/pages/search.astro.mjs","/_worker.js/pages/status.astro.mjs","/_worker.js/chunks/Layout_BGy5vrpZ.mjs","/_worker.js/chunks/_@astro-renderers_B2-kUL1M.mjs","/_worker.js/chunks/_@astrojs-ssr-adapter_jipRO4xr.mjs","/_worker.js/chunks/astro-designed-error-pages_DTrXhg6j.mjs","/_worker.js/chunks/astro_BbzGv9mj.mjs","/_worker.js/chunks/cloudflare-kv-binding_DMly_2Gl.mjs","/_worker.js/chunks/generic_YXfy00V2.mjs","/_worker.js/chunks/index_ACNsl3_t.mjs","/_worker.js/chunks/noop-middleware_CMcsHAPi.mjs","/_worker.js/chunks/noop_Desx2XLg.mjs","/_worker.js/chunks/path_CH3auf61.mjs","/_worker.js/chunks/remote_CrdlObHx.mjs","/_worker.js/pages/api/_---path_.astro.mjs","/_worker.js/chunks/astro/server_BTqzngb8.mjs"],"buildFormat":"directory","checkOrigin":true,"allowedDomains":[],"actionBodySizeLimit":1048576,"serverIslandNameMap":[],"key":"21gDH2WNNtGANCBjH1KFZza7Z8yjh+BgyMOwz2PPJ7k=","sessionConfig":{"driver":"cloudflare-kv-binding","options":{"binding":"SESSION"}}});
if (manifest.sessionConfig) manifest.sessionConfig.driverModule = () => import('./chunks/cloudflare-kv-binding_DMly_2Gl.mjs');

export { manifest };
