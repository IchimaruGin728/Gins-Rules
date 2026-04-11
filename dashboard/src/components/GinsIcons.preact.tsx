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
  const [copiedUrl, setCopiedUrl] = useState<string | null>(null);

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
    setCopiedUrl(url);
    setTimeout(() => setCopiedUrl(null), 2000);
  };

  return (
    <div class="flex flex-col gap-6 animate-in fade-in duration-500">
      {/* Header & Controls */}
      <div class="glass-panel p-6! flex flex-col gap-6">
        <div class="flex flex-col md:flex-row gap-6 items-center justify-between">
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
              class="bg-white/5 border border-white/10 rounded-2xl px-6 py-3 text-[11px] font-black tracking-widest text-white focus:outline-none focus:border-brand-primary/50 focus:bg-white/[0.08] transition-all uppercase appearance-none cursor-pointer"
            >
              {sources.map((s) => (
                <option value={s}>{s.toUpperCase()}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Subscription URL Bar (Standard Size) */}
        <div class="flex flex-col sm:flex-row items-center gap-4 pt-6 border-t border-white/5">
          <div class="flex-1 flex items-center gap-3 w-full bg-black/20 border border-white/5 rounded-2xl px-4 py-2.5 group hover:border-brand-primary/30 transition-colors">
            <div class="i-ph-link-bold text-brand-primary text-sm"></div>
            <code class="text-[10px] font-mono text-gray-400 select-all truncate">https://rules.ichimarugin728.dev/Gins-Icons.json</code>
          </div>
          <button 
            onClick={() => copyToClipboard("https://rules.ichimarugin728.dev/Gins-Icons.json")}
            class="w-full sm:w-auto px-6 py-2.5 bg-brand-primary/10 hover:bg-brand-primary/20 border border-brand-primary/30 rounded-xl text-[10px] font-black text-brand-primary uppercase tracking-widest transition-all active:scale-95 flex items-center justify-center gap-2"
          >
            {copiedUrl === "https://rules.ichimarugin728.dev/Gins-Icons.json" ? <div class="i-ph-check-bold"></div> : <div class="i-ph-copy-bold"></div>}
            {copiedUrl === "https://rules.ichimarugin728.dev/Gins-Icons.json" ? "COPIED" : "COPY JSON URL"}
          </button>
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

            {/* Always Visible Copy Indicator (Small) */}
            <div class="absolute top-3 right-3 flex items-center justify-center w-6 h-6 rounded-lg bg-white/5 border border-white/5 group-hover:border-brand-primary/30 group-hover:bg-brand-primary/10 transition-all opacity-40 group-hover:opacity-100">
               <div class={`text-[10px] ${copiedUrl === icon.url ? 'i-ph-check-bold text-green-400' : 'i-ph-copy-simple-bold text-brand-primary'}`}></div>
            </div>

            {/* Copy Feedback Overlay (Center) */}
            <div class={`absolute inset-0 flex items-center justify-center bg-brand-primary/90 transition-all duration-300 ${copiedUrl === icon.url ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}>
              <div class="flex flex-col items-center gap-2">
                <div class="i-ph-check-circle-fill text-3xl text-white"></div>
                <span class="text-[10px] font-black text-white uppercase tracking-widest">COPIED!</span>
              </div>
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

