import { activeApp, APPS, optimizedDomainSet } from "../store.preact";
import AppLogo from "./AppLogo.preact";

export default function AppSwitcher() {
  const showOptimizedToggle = ["surfboard", "surge", "shadowrocket"].includes(activeApp.value);
  const extension = activeApp.value === "surge" ? ".domainset" : ".txt";

  return (
    <div class="flex flex-col gap-4">
      <div class="glass-panel p-1.5! flex flex-wrap gap-1.5 items-center justify-center sm:justify-start overflow-hidden">
        {APPS.map((app) => (
          <button
            type="button"
            onClick={() => (activeApp.value = app.id)}
            class={`
              flex items-center gap-2 px-3 py-2 rounded-xl border transition-all duration-300 active:scale-95
              ${
                activeApp.value === app.id
                  ? `bg-white/[0.08] border-white/20 shadow-lg text-white`
                  : `bg-transparent border-transparent text-gray-500 hover:text-gray-300 hover:bg-white/[0.02]`
              }
            `}
          >
            <AppLogo
              icon={app.icon}
              accent={app.color}
              size="sm"
              active={activeApp.value === app.id}
              class="transition-transform duration-300"
            />
            <span
              class={`text-[11px] font-black uppercase tracking-widest ${
                activeApp.value === app.id ? "opacity-100" : "opacity-60"
              }`}
            >
              {app.label}
            </span>

            {activeApp.value === app.id && (
              <div
                class="w-1 h-1 rounded-full animate-ping"
                style={{ backgroundColor: app.color }}
              ></div>
            )}
          </button>
        ))}
        
        {/* Divider */}
        <div class="w-px h-6 bg-white/10 mx-2 hidden sm:block"></div>

        {/* Icon Hub Link */}
        <a
          href="/icons"
          class="flex items-center gap-2 px-3 py-2 rounded-xl border border-transparent text-brand-primary hover:text-white hover:bg-brand-primary/20 transition-all duration-300 group"
        >
          <div class="i-ph-shapes-duotone text-lg group-hover:rotate-12 transition-transform"></div>
          <span class="text-[11px] font-black uppercase tracking-widest">
            Icon Hub
          </span>
        </a>
      </div>

      {showOptimizedToggle && (
        <div class="flex items-center gap-4 px-2 animate-in fade-in slide-in-from-top-2 duration-300">
          <div class="i-ph-arrow-bend-down-right-bold text-gray-600 text-xl ml-4"></div>
          <button
            onClick={() => (optimizedDomainSet.value = !optimizedDomainSet.value)}
            class={`
              flex items-center gap-3 px-4 py-2.5 rounded-2xl border transition-all duration-300
              ${
                optimizedDomainSet.value
                  ? "bg-brand-primary/10 border-brand-primary/30 text-brand-primary"
                  : "bg-white/5 border-white/10 text-gray-500 hover:border-white/20"
              }
            `}
          >
            <div
              class={`text-lg ${
                optimizedDomainSet.value ? "i-ph-check-square-fill" : "i-ph-square-bold"
              }`}
            ></div>
            <span class="text-[10px] font-black uppercase tracking-[0.2em]">
              Enable Domain Set ({extension})
            </span>
          </button>
          <p class="text-[10px] text-gray-600 font-bold uppercase tracking-wider italic">
            Optimize for high-performance domain matching
          </p>
        </div>
      )}
    </div>
  );
}
