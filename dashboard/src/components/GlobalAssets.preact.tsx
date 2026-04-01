import { activeApp } from "../store.preact";
import DatCard from "./DatCard.preact";

interface GlobalAssetsProps {
  apiBase: string;
}

export default function GlobalAssets({ apiBase }: GlobalAssetsProps) {
  const showDat = ["v2ray", "exclave"].includes(activeApp.value);
  
  return (
    <div class="space-y-8">
      {/* DAT Section - Conditional */}
      {showDat && (
        <section class="space-y-4 animate-in fade-in duration-500">
          <div class="flex items-center gap-2 px-1">
            <div class="i-ph-package-bold text-brand-primary"></div>
            <span class="text-[10px] font-black uppercase tracking-[0.3em] text-white/40">
              V2Ray Routing Assets (Binary DAT)
            </span>
          </div>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <DatCard name="geosite.dat" url={`${apiBase}/ruleset/xray/geosite.dat`} />
            <DatCard name="geoip.dat" url={`${apiBase}/ruleset/xray/geoip.dat`} />
            <DatCard name="geoasn.dat" url={`${apiBase}/ruleset/xray/geoasn.dat`} />
          </div>
        </section>
      )}

      {/* MMDB Section - Always Visible for Unified distribution */}
      <section class="space-y-4">
        <div class="flex items-center gap-2 px-1">
          <div class="i-ph-database-bold text-brand-secondary"></div>
          <span class="text-[10px] font-black uppercase tracking-[0.3em] text-white/40">
            Advanced Geo Databases (MMDB / MAXMIND)
          </span>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <DatCard name="geoip.mmdb" url={`${apiBase}/ruleset/geoip.mmdb`} />
          <DatCard name="geoasn.mmdb" url={`${apiBase}/ruleset/geoasn.mmdb`} />
        </div>
      </section>
    </div>
  );
}
