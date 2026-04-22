import { useState } from "preact/hooks";
import { activeApp } from "../store.preact";

interface Props {
  apiBase: string;
}

export default function GeoChecker({ apiBase }: Props) {
  const [copied, setCopied] = useState(false);

  if (activeApp.value !== "quantumultx") return null;

  const codeString = `geo_location_checker = https://my.ippure.com/v1/info, ${apiBase}/geo_location_checker.js`;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(codeString);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Copy failed", err);
    }
  };

  return (
    <div class="glass-panel p-6! space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
      <div class="flex items-center gap-2">
        <div class="i-ph-map-pin-line-bold text-[#f44336]"></div>
        <span class="text-[10px] font-black uppercase tracking-[0.3em] text-white/40">
          Node Geo Location Checker
        </span>
      </div>
      
      <p class="text-xs text-gray-400">
        Injects a beautifully-formatted, pure English IP risk and location UI into your Quantumult X nodes panel.
      </p>

      <button
        type="button"
        onClick={handleCopy}
        class={`
          group relative flex items-center justify-between w-full p-4 rounded-xl transition-all duration-300
          bg-white/[0.03] border border-[#f44336]/30 hover:bg-white/[0.06] hover:border-[#f44336]/60
          ${copied ? "bg-[#f44336]/10! border-[#f44336]!" : ""}
          active:scale-[0.98] mt-2
        `}
      >
        <div class="flex flex-col items-start w-full pr-12">
          <code class="text-left font-mono text-xs text-brand-primary break-all">
            {codeString}
          </code>
        </div>
        
        <div
          class={`
          absolute right-4 p-2 rounded-lg border transition-all duration-300
          ${
            copied
              ? "bg-[#f44336] border-[#f44336] text-white scale-110"
              : "bg-white/5 border-white/5 text-gray-500 group-hover:text-[#f44336] group-hover:border-[#f44336]/20 group-hover:bg-[#f44336]/5"
          }
        `}
        >
          <div class={`${copied ? "i-ph-check-bold" : "i-ph-copy-bold"} text-xs`}></div>
        </div>

        {copied && (
          <div class="absolute -top-3 right-2 px-2 py-1 bg-[#f44336] text-white text-[8px] font-black uppercase tracking-widest rounded shadow-lg animate-bounce">
            Copied
          </div>
        )}
      </button>
    </div>
  );
}
