import { activeApp, APPS } from "../store.preact";
import DynamicIcon from "./DynamicIcon.preact";

export default function AppSwitcher() {
  return (
    <div class="glass-panel !p-1.5 flex flex-wrap gap-1.5 items-center justify-center sm:justify-start overflow-hidden">
      {APPS.map((app) => (
        <button
          onClick={() => (activeApp.value = app.id)}
          class={`
            flex items-center gap-2 px-3 py-2 rounded-xl border transition-all duration-300 active:scale-95
            ${activeApp.value === app.id 
              ? `bg-white/[0.08] border-white/20 shadow-lg text-white` 
              : `bg-transparent border-transparent text-gray-500 hover:text-gray-300 hover:bg-white/[0.02]`
            }
          `}
        >
          <DynamicIcon 
            icon={app.icon}
            class={`text-lg w-4.5 h-4.5 transition-transform duration-300 ${activeApp.value === app.id ? 'scale-110' : 'scale-100'}`}
            style={activeApp.value === app.id ? { color: app.color } : {}}
          />
          <span class={`text-[11px] font-black uppercase tracking-widest ${activeApp.value === app.id ? 'opacity-100' : 'opacity-60'}`}>
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
    </div>
  );
}
