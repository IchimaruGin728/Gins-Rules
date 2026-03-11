import { useState } from 'preact/hooks';

interface Props {
  name: string;
  category: string;
  lines: number;
  apiBase: string;
}

export default function ServiceItem({ name, category, lines, apiBase }: Props) {
  const [copied, setCopied] = useState<string | null>(null);
  const cleanName = name.replace('.txt', '');

  const formats = [
    { id: 'srs', icon: 'i-ph-cube-bold', tooltip: 'Sing-box' },
    { id: 'mrs', icon: 'i-ph-navigation-arrow-bold', tooltip: 'Mihomo' },
    { id: 'list', icon: 'i-ph-file-text-bold', tooltip: 'Text List' },
  ];

  const handleCopy = async (ext: string, e: any) => {
    // Prevent standard click behavior or text selection
    e.preventDefault();
    e.stopPropagation();

    const url = `${apiBase}/ruleset/${category}/${cleanName}.${ext}`;
    try {
      await navigator.clipboard.writeText(url);
      setCopied(ext);
      setTimeout(() => setCopied(null), 1000);
    } catch (err) {
      console.error('Copy failed', err);
    }
  };

  return (
    <div 
      class={`
        group/item relative flex flex-col items-start p-3 rounded-2xl transition-all duration-300
        bg-white/[0.04] border border-white/[0.06] hover:bg-white/[0.08] hover:border-white/20
        ${copied ? '!bg-brand-primary/10 !border-brand-primary/30' : ''}
      `}
    >
      <div class="flex items-center justify-between w-full mb-2">
        <span class="text-gray-200 font-mono text-[11px] font-bold truncate pr-2 group-hover/item:text-brand-primary transition-colors">
          {cleanName}
        </span>
        <span class="text-[9px] text-gray-500 font-black uppercase tracking-tighter opacity-60">
          {lines} L
        </span>
      </div>

      <div class="flex gap-1.5 w-full">
        {formats.map((f) => (
          <button
            title={f.tooltip}
            onClick={(e) => handleCopy(f.id, e)}
            class={`
              flex-1 flex items-center justify-center py-1.5 rounded-lg border transition-all active:scale-90
              ${copied === f.id 
                ? 'bg-brand-primary text-white border-brand-primary shadow-lg shadow-brand-primary/20' 
                : 'bg-white/5 text-gray-500 border-white/5 hover:bg-white/10 hover:text-brand-primary hover:border-brand-primary/20'
              }
            `}
          >
            <div class={`${copied === f.id ? 'i-ph-check-bold scale-110' : f.icon} text-[10px] transition-all`}></div>
          </button>
        ))}
      </div>

      {/* Inline Feedback Toast */}
      {copied && (
        <div class="absolute -top-2 -right-1 px-2 py-0.5 bg-brand-primary text-white text-[8px] font-black uppercase tracking-widest rounded-full animate-bounce shadow-lg">
          Copied
        </div>
      )}
    </div>
  );
}
