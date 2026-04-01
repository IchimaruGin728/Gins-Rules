import { useState } from "preact/hooks";

export default function DatCard({ name, url }: { name: string; url: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy", err);
    }
  };

  return (
    <div class="glass-panel p-4! flex flex-col gap-4 group/dat relative overflow-hidden">
      <div class="absolute top-0 right-0 p-4 opacity-5 pointer-events-none group-hover/dat:opacity-10 transition-opacity">
        <div class="i-ph-file-zip-duotone text-5xl"></div>
      </div>

      <div class="flex items-center justify-between relative">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-xl bg-brand-primary/10 border border-brand-primary/20 flex items-center justify-center">
            <div class="i-ph-package-bold text-brand-primary"></div>
          </div>
          <div>
            <div class="font-outfit font-black text-sm tracking-tight text-white/90">
              {name}
            </div>
            <div class="text-[8px] font-black uppercase tracking-widest text-gray-500 mt-0.5">
              {name.endsWith(".dat") ? "V2Ray / Xray Binary" : "MaxMind Database / GeoIP"}
            </div>
          </div>
        </div>
      </div>

      <div class="flex gap-2 relative">
        <a
          href={url}
          target="_blank"
          class="flex-1 px-3 py-2.5 rounded-xl bg-white/5 border border-white/10 hover:bg-brand-primary/10 hover:border-brand-primary/30 text-[9px] font-black uppercase tracking-[0.2em] text-center transition-all duration-300"
        >
          Download
        </a>
        <button
          onClick={handleCopy}
          class={`
            px-3 py-2.5 rounded-xl border transition-all duration-300
            ${
              copied
                ? "bg-green-500/20 border-green-500/40 text-green-400"
                : "bg-white/5 border-white/10 hover:bg-white/10 text-white/60 hover:text-white"
            }
          `}
        >
          <div class={`text-sm ${copied ? "i-ph-check-bold" : "i-ph-copy-bold"}`}></div>
        </button>
      </div>
    </div>
  );
}
