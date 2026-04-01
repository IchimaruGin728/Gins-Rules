import type { JSX } from "preact";
import { useState } from "preact/hooks";
import { activeApp, getActiveConfig, surfboardDomainSet } from "../store.preact";

interface Props {
  name: string;
  category: string;
  lines: number;
  apiBase: string;
}

export default function ServiceItem({ name, category, lines, apiBase }: Props) {
  const [copied, setCopied] = useState(false);
  const cleanName = name.replace(".txt", "");
  const config = getActiveConfig();

  const handleCopy = async (e: JSX.TargetedMouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    e.stopPropagation();

    // Surfboard Domain Set logic
    const ext = (activeApp.value === "surfboard" && surfboardDomainSet.value) ? "txt" : config.ext;
    
    // URL Format: ruleset/:app/:category/:name.ext
    const url = `${apiBase}/ruleset/${activeApp.value}/${category}/${cleanName}.${ext}`;

    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch (err) {
      console.error("Copy failed", err);
    }
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      class={`
        group/item relative flex items-center justify-between w-full p-3 rounded-xl transition-all duration-300
        bg-white/[0.03] border border-white/[0.04] hover:bg-white/[0.06] hover:border-white/20
        ${copied ? "bg-brand-primary/10! border-brand-primary/30!" : ""}
        active:scale-[0.98]
      `}
    >
      <div class="flex items-center gap-3 overflow-hidden">
        <div
          class="w-1.5 h-1.5 rounded-full transition-transform duration-500 group-hover/item:scale-150"
          style={{ backgroundColor: config.color }}
        ></div>
        <div class="flex flex-col items-start overflow-hidden">
          <span class="text-gray-300 font-mono text-[11px] font-bold truncate w-full group-hover/item:text-white transition-colors">
            {cleanName}
          </span>
          <span class="text-[8px] text-gray-500 font-black uppercase tracking-widest mt-0.5">
            {lines} Rules
          </span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div
          class={`
          rounded-[1.1rem] border p-1 transition-all duration-300
          ${
            copied
              ? "bg-brand-primary border-brand-primary text-white scale-110 shadow-lg shadow-brand-primary/20"
              : "bg-white/5 border-white/5 text-gray-500 group-hover/item:border-brand-primary/20 group-hover/item:bg-brand-primary/5"
          }
        `}
        >
          {copied ? (
            <div class="i-ph-check-bold text-xs transition-transform duration-500"></div>
          ) : (
            <div class="i-ph-copy-bold text-xs transition-transform duration-500 group-hover/item:scale-105"></div>
          )}
        </div>
      </div>

      {/* Mini Feedback */}
      {copied && (
        <div class="absolute -top-1 -right-1 px-1.5 py-0.5 bg-brand-primary text-white text-[7px] font-black uppercase tracking-widest rounded-md shadow-lg animate-bounce">
          Copied
        </div>
      )}
    </button>
  );
}
