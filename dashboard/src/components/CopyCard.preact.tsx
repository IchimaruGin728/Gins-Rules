import { useState } from 'preact/hooks';

interface Props {
  label: string;
  category: string;
  icon: string;
  baseUrl: string;
}

export default function CopyCard({ label, category, icon, baseUrl }: Props) {
  const [copied, setCopied] = useState<string | null>(null);

  const formats = [
    { label: 'Sing-box', ext: 'srs', icon: 'i-ph-cube-bold' },
    { label: 'Mihomo', ext: 'mrs', icon: 'i-ph-navigation-arrow-bold' },
    { label: 'Text List', ext: 'list', icon: 'i-ph-file-text-bold' },
    { label: 'Egern', ext: 'yaml', icon: 'i-ph-scroll-bold' },
  ];

  const handleCopy = async (ext: string, e: MouseEvent) => {
    e.stopPropagation();
    const url = `${baseUrl}/ruleset/${category}.${ext}`;
    try {
      await navigator.clipboard.writeText(url);
      setCopied(ext);
      setTimeout(() => setCopied(null), 2000);
    } catch (err) {
      console.error('Failed to copy!', err);
    }
  };

  return (
    <div class="glass-panel group relative overflow-hidden transition-all duration-500 hover:scale-[1.02] hover:shadow-2xl hover:shadow-brand-primary/10">
      {/* Background Glow */}
      <div class="absolute -inset-1 bg-gradient-to-r from-brand-primary/0 via-brand-primary/5 to-brand-primary/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 blur"></div>
      
      <div class="relative p-6 flex flex-col h-full">
        <div class="flex items-center gap-4 mb-6">
          <div class={`${icon} text-3xl text-brand-primary group-hover:scale-110 transition-transform duration-500`}></div>
          <div>
            <h3 class="font-outfit font-black text-xl tracking-tight text-white group-hover:text-brand-primary transition-colors">{label}</h3>
            <p class="text-[10px] font-bold uppercase tracking-[0.2em] text-gray-500">Ruleset Bundle</p>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2 mt-auto">
          {formats.map((f) => (
            <button
              onClick={(e) => handleCopy(f.ext, e)}
              class={`
                relative flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-bold transition-all duration-300
                ${copied === f.ext 
                  ? 'bg-green-500/20 text-green-400 border-green-500/30' 
                  : 'bg-white/5 text-gray-400 border-white/5 hover:bg-white/10 hover:text-white hover:border-white/10'
                }
                border active:scale-95
              `}
            >
              <div class={`${copied === f.ext ? 'i-ph-check-circle-bold scale-110' : f.icon} transition-all`}></div>
              <span>{copied === f.ext ? 'Copied!' : f.label}</span>
              
              {/* Particle Burst Effect on Copy */}
              {copied === f.ext && (
                <div class="absolute inset-0 pointer-events-none animate-ping opacity-50 bg-green-500/20 rounded-xl"></div>
              )}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
