import { useState } from "preact/hooks";
import { activeApp, getActiveConfig, surfboardDomainSet } from "../store.preact";
import AppLogo from "./AppLogo.preact";

interface Props {
  label: string;
  category: string;
  icon: string;
  baseUrl: string;
}

export default function CopyCard({ label, category, icon, baseUrl }: Props) {
  const [copied, setCopied] = useState(false);
  const config = getActiveConfig();

  const handleCopy = async () => {
    // Surfboard Domain Set logic
    const ext = (activeApp.value === "surfboard" && surfboardDomainSet.value) ? "txt" : config.ext;

    // URL Format: ruleset/:app/:category.ext
    const url = `${baseUrl}/ruleset/${activeApp.value}/${category}.${ext}`;

    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Copy failed", err);
    }
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      class={`
        group relative w-full overflow-hidden glass-panel p-8! flex flex-col items-center justify-center gap-5 transition-all duration-500
        ${copied ? "border-brand-primary/40!" : "hover:border-white/20 hover:scale-[1.02]"}
        active:scale-[0.98]
      `}
    >
      {/* Dynamic Background Glow */}
      <div
        class="absolute -inset-20 opacity-0 group-hover:opacity-10 transition-opacity duration-1000 blur-[80px] pointer-events-none"
        style={{ backgroundColor: config.color }}
      ></div>

      <div class="relative flex items-center justify-center p-5 rounded-[2.5rem] bg-white/[0.03] border border-white/5 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
        <div
          class={`${icon} text-4xl transition-colors duration-500`}
          style={{ color: config.color }}
        ></div>
      </div>

      <div class="relative text-center">
        <div class="font-outfit font-black text-2xl tracking-tighter text-white group-hover:text-brand-primary transition-colors">
          {label}
        </div>
        <div class="flex items-center gap-2.5 justify-center mt-3">
          <AppLogo icon={config.icon} accent={config.color} size="sm" active />
          <span class="text-[10px] font-black uppercase tracking-[0.3em] text-gray-500 group-hover:text-gray-400 transition-colors">
            {config.label} Link
          </span>
        </div>
      </div>

      {/* Action Overlay */}
      <div
        class={`
        absolute inset-0 flex items-center justify-center bg-black/40 backdrop-blur-md transition-all duration-500
        ${copied ? "opacity-100 scale-100" : "opacity-0 scale-90 pointer-events-none"}
      `}
      >
        <div class="flex flex-col items-center gap-3">
          <div class="i-ph-check-circle-fill text-5xl text-brand-primary"></div>
          <span class="font-outfit font-black text-sm uppercase tracking-[0.4em] text-brand-primary animate-pulse">
            Copied!
          </span>
        </div>
      </div>
    </button>
  );
}
