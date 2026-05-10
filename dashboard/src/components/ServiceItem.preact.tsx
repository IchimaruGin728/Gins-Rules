import type { JSX } from "preact";
import { useState } from "preact/hooks";
import {
  activeApp,
  getActiveConfig,
  optimizedDomainSet,
} from "../store.preact";

interface Props {
  name: string;
  category: string;
  lines: number;
  apiBase: string;
  classical?: boolean;
  pureDomain?: boolean;
  hasIP?: boolean;
}

export default function ServiceItem({
  name,
  category,
  lines,
  apiBase,
  classical,
  pureDomain,
  hasIP,
}: Props) {
  const [copied, setCopied] = useState(false);
  const [yamlCopied, setYamlCopied] = useState(false);
  const cleanName = name.replace(".txt", "");
  const config = getActiveConfig();

  const isMihomoLike = ["mihomo", "stash"].includes(activeApp.value);
  const isIPPart = name.includes("-ip");

  // Logic: Non-Mihomo apps don't use split -ip files
  if (isIPPart && !isMihomoLike) return null;

  // DOMAIN-SET STRICTURE: Must be pure domain (no IP, no classical rules like regex/keywords)
  const isDomainSetSupported = pureDomain;
  const isDisabled =
    optimizedDomainSet.value &&
    !isDomainSetSupported &&
    category !== "ip" &&
    category !== "asn";

  const handleCopy = async (
    e: JSX.TargetedMouseEvent<HTMLButtonElement>,
    forceYaml = false,
  ) => {
    if (isDisabled) return;
    e.preventDefault();
    e.stopPropagation();

    const isIPCategory = category === "ip" || category === "asn";
    let ext = config.ext;

    if (optimizedDomainSet.value && !isIPCategory) {
      if (activeApp.value === "surge") ext = "domainset";
      else if (
        activeApp.value === "surfboard" ||
        activeApp.value === "shadowrocket"
      )
        ext = "txt";
    }

    // Manual YAML override
    if (forceYaml) {
      ext = "yaml";
    }

    const url = `${apiBase}/ruleset/${activeApp.value}/${category}/${cleanName}.${ext}`;

    try {
      await navigator.clipboard.writeText(url);
      if (forceYaml) {
        setYamlCopied(true);
        setTimeout(() => setYamlCopied(false), 1500);
      } else {
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      }
    } catch (err) {
      console.error("Copy failed", err);
    }
  };

  const showIPBadges = hasIP && (!isMihomoLike || isIPPart);

  return (
    <div
      class={`
        group/item relative flex items-center justify-between w-full p-3 rounded-xl transition-all duration-300
        ${
          isDisabled
            ? "bg-white/[0.01] border border-white/5 opacity-40 cursor-not-allowed grayscale"
            : "bg-white/[0.03] border border-white/[0.04] hover:bg-white/[0.06] hover:border-white/20"
        }
        ${copied || yamlCopied ? "border-brand-primary/40!" : ""}
      `}
    >
      <div class="flex items-center gap-3 overflow-hidden">
        <div
          class="w-1.5 h-1.5 rounded-full transition-transform duration-500 group-hover/item:scale-150"
          style={{ backgroundColor: isDisabled ? "#444" : config.color }}
        ></div>
        <div class="flex flex-col items-start overflow-hidden text-left">
          <div class="flex items-center gap-1.5 truncate w-full">
            <span class="text-gray-300 font-mono text-[11px] font-bold truncate group-hover/item:text-white transition-colors">
              {cleanName}
            </span>
            {showIPBadges && !isDisabled && (
              <div class="flex gap-0.5">
                <span
                  class="text-[7px] px-1 py-0 bg-white/10 text-white/60 rounded font-black uppercase tracking-tighter"
                  title="Includes IP CIDR rules"
                >
                  IP
                </span>
                <span
                  class="text-[7px] px-1 py-0 bg-brand-primary/20 text-brand-primary rounded font-black uppercase tracking-tighter cursor-help"
                  title="Contains IP rules. Enable 'no-resolve' in your config to avoid redundant DNS lookups."
                >
                  NR
                </span>
              </div>
            )}
          </div>
          <span class="text-[8px] text-gray-500 font-black uppercase tracking-widest mt-0.5">
            {isDisabled ? "Unsupported Domain-Set" : `${lines} Rules`}
          </span>
        </div>
      </div>

      <div class="flex items-center gap-1.5">
        {/* Classical YAML Fallback Button for Mihomo/Stash */}
        {isMihomoLike && classical && !isDisabled && (
          <button
            onClick={(e) => handleCopy(e, true)}
            title="Copy full Classical YAML"
            class={`text-[9px] font-black px-1.5 py-1 rounded-md border transition-all duration-300 ${
              yamlCopied
                ? "bg-brand-secondary border-brand-secondary text-white"
                : "bg-white/5 border-white/10 text-white/40 hover:text-white hover:border-white/30"
            }`}
          >
            {yamlCopied ? "OK" : "YAML"}
          </button>
        )}

        <button
          type="button"
          onClick={(e) => handleCopy(e)}
          disabled={isDisabled}
          class={`
          rounded-[1.1rem] border p-1 transition-all duration-300
          ${
            copied
              ? "bg-brand-primary border-brand-primary text-white scale-110 shadow-lg shadow-brand-primary/20"
              : isDisabled
                ? "bg-transparent border-transparent text-white/5"
                : "bg-white/5 border-white/5 text-gray-500 group-hover/item:border-brand-primary/20 group-hover/item:bg-brand-primary/5"
          }
        `}
        >
          {copied ? (
            <div class="i-ph-check-bold text-xs"></div>
          ) : (
            <div
              class={`text-xs ${isDisabled ? "i-ph-prohibit-bold" : "i-ph-copy-bold"}`}
            ></div>
          )}
        </button>
      </div>

      {(copied || yamlCopied) && (
        <div class="absolute -top-1 -right-1 px-1.5 py-0.5 bg-brand-primary text-white text-[7px] font-black uppercase tracking-widest rounded-md shadow-lg animate-bounce">
          Copied
        </div>
      )}
    </div>
  );
}
