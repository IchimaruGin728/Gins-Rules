import { useState, useMemo } from "preact/hooks";

interface Icon {
  name: string;
  url: string;
  source: string;
  theme: string;
}

export default function GinsIcons({ initialIcons }: { initialIcons: Icon[] }) {
  const [search, setSearch] = useState("");
  const [activeSource, setActiveSource] = useState("All");

  const sources = useMemo(() => {
    const s = new Set(initialIcons.map((i) => i.source));
    return ["All", ...Array.from(s).sort()];
  }, [initialIcons]);

  const filteredIcons = useMemo(() => {
    return initialIcons.filter((icon) => {
      const matchSearch = icon.name.toLowerCase().includes(search.toLowerCase());
      const matchSource = activeSource === "All" || icon.source === activeSource;
      return matchSearch && matchSource;
    });
  }, [initialIcons, search, activeSource]);

  const copyToClipboard = (url: string) => {
    navigator.clipboard.writeText(url);
    // Simple toast could be added here
  };

  return (
    <div class="flex flex-col gap-8 animate-in fade-in duration-500">
      {/* Header & Controls */}
      <div class="glass-panel p-6! flex flex-col md:flex-row gap-6 items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-2xl bg-brand-primary/20 flex items-center justify-center border border-brand-primary/30 shadow-[0_0_20px_rgba(var(--brand-primary-rgb),0.2)]">
            <div class="i-ph-shapes-bold text-2xl text-brand-primary"></div>
          </div>
          <div>
            <h1 class="text-xl font-black tracking-tight text-white uppercase">Gins Icon Hub</h1>
            <p class="text-[10px] text-gray-500 font-bold uppercase tracking-widest mt-0.5">Automated Multi-Source Library</p>
          </div>
        </div>

        <div class="flex flex-col sm:flex-row gap-4 w-full md:w-auto">
          {/* Search */}
          <div class="relative group">
            <div class="absolute left-4 top-1/2 -translate-y-1/2 i-ph-magnifying-glass-bold text-gray-500 group-focus-within:text-brand-primary transition-colors"></div>
            <input
              type="text"
              placeholder="SEARCH ICONS..."
              value={search}
              onInput={(e) => setSearch(e.currentTarget.value)}
              class="bg-white/5 border border-white/10 rounded-2xl pl-11 pr-6 py-3 text-[11px] font-black tracking-widest text-white focus:outline-none focus:border-brand-primary/50 focus:bg-white/[0.08] transition-all w-full sm:w-64 uppercase"
            />
          </div>

          {/* Source Selector */}
          <select
            value={activeSource}
            onChange={(e) => setActiveSource(e.currentTarget.value)}
            class="bg-white/5 border border-white/10 rounded-2xl px-6 py-3 text-[11px] font-black tracking-widest text-white focus:outline-none focus:border-brand-primary/50 focus:bg-white/[0.08] transition-all uppercase appearance-none"
          >
            {sources.map((s) => (
              <option value={s}>{s.toUpperCase()}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Grid */}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {filteredIcons.map((icon) => (
          <div
            key={icon.url}
            onClick={() => copyToClipboard(icon.url)}
            class="glass-panel group cursor-pointer hover:border-brand-primary/40 hover:bg-white/[0.05] transition-all duration-300 p-4 flex flex-col items-center justify-center gap-4 relative overflow-hidden active:scale-95"
          >
            {/* Glossy Overlay */}
            <div class="absolute inset-0 bg-gradient-to-tr from-transparent via-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
            
            <img
              src={icon.url}
              alt={icon.name}
              loading="lazy"
              class="w-16 h-16 object-contain drop-shadow-xl group-hover:scale-110 transition-transform duration-500"
            />
            
            <div class="text-center w-full">
              <p class="text-[10px] font-black text-gray-300 uppercase truncate px-2 group-hover:text-white transition-colors">
                {icon.name}
              </p>
              <p class="text-[8px] font-bold text-gray-600 uppercase tracking-tighter mt-1 opacity-60">
                {icon.source}
              </p>
            </div>

            {/* Copy Badge */}
            <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
              <div class="i-ph-copy-simple-bold text-brand-primary text-xs"></div>
            </div>
          </div>
        ))}
      </div>

      {filteredIcons.length === 0 && (
        <div class="glass-panel p-20 flex flex-col items-center justify-center text-center gap-4">
          <div class="i-ph-ghost-bold text-6xl text-gray-700 animate-bounce"></div>
          <p class="text-gray-500 font-bold uppercase tracking-[0.3em] text-xs">No Icons Found</p>
        </div>
      )}
    </div>
  );
}
