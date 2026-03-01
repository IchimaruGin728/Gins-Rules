globalThis.process ??= {}; globalThis.process.env ??= {};
import { l } from './_@astro-renderers_B2-kUL1M.mjs';

var f=0;function u(e,t,n,o,i,u){t||(t={});var a,c,p=t;if("ref"in p)for(c in p={},t)"ref"==c?a=t[c]:p[c]=t[c];var l$1={type:e,props:p,key:n,ref:a,__k:null,__:null,__b:0,__e:null,__c:null,constructor:void 0,__v:--f,__i:-1,__u:0,__source:i,__self:u};if("function"==typeof e&&(a=e.defaultProps))for(c in a) void 0===p[c]&&(p[c]=a[c]);return l.vnode&&l.vnode(l$1),l$1}

function Layout({
  children,
  title
}) {
  const pageTitle = title ? `${title} | Gins-Rules` : "Gins-Rules Dashboard";
  return u("div", {
    class: "min-h-screen",
    "data-title": pageTitle,
    children: [u("div", {
      class: "bg-glow"
    }), u("header", {
      class: "fixed top-6 left-0 right-0 z-50 flex justify-center px-4",
      children: u("nav", {
        class: "nav-pill",
        children: [u("div", {
          class: "flex items-center gap-2 mr-4",
          children: [u("div", {
            class: "i-ph-shield-check-fill text-2xl text-brand-primary"
          }), u("span", {
            class: "text-xl font-extrabold font-outfit text-gradient tracking-tight",
            children: "Gins"
          })]
        }), u("div", {
          class: "flex items-center gap-6 text-sm font-medium text-gray-400",
          children: [u("a", {
            href: "/",
            class: "hover:text-white transition-colors duration-300 flex items-center gap-1.5",
            children: [u("div", {
              class: "i-ph-house-bold"
            }), " Home"]
          }), u("a", {
            href: "/search",
            class: "hover:text-white transition-colors duration-300 flex items-center gap-1.5",
            children: [u("div", {
              class: "i-ph-magnifying-glass-bold"
            }), " Search"]
          }), u("a", {
            href: "/ai",
            class: "hover:text-white transition-colors duration-300 flex items-center gap-1.5",
            children: [u("div", {
              class: "i-ph-sparkle-bold"
            }), " AI Monitor"]
          }), u("a", {
            href: "/status",
            class: "hover:text-white transition-colors duration-300 flex items-center gap-1.5",
            children: [u("div", {
              class: "i-ph-pulse-bold"
            }), " Status"]
          })]
        })]
      })
    }), u("main", {
      class: "max-w-7xl mx-auto pt-32 p-6 md:p-8 animate-fade-in-up",
      children
    }), u("footer", {
      class: "mt-20 py-12 border-t border-white/5 text-center text-gray-600 text-sm font-medium tracking-wide",
      children: u("p", {
        children: "© 2026 GINS-RULES. POWERED BY CLOUDFLARE EDGE."
      })
    })]
  });
}

export { Layout as L };
