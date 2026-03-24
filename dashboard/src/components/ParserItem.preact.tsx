import { useState } from 'preact/hooks';

interface Props {
  name: string;
  url: string;
  icon: string;
}

export default function ParserItem({ name, url, icon }: Props) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Copy failed', err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      class={`
        group relative flex items-center justify-between w-full p-4 rounded-xl transition-all duration-300
        bg-white/[0.03] border border-white/[0.04] hover:bg-white/[0.06] hover:border-white/20
        ${copied ? '!bg-brand-primary/10 !border-brand-primary/30' : ''}
        active:scale-[0.98]
      `}
    >
      <div class="flex items-center gap-3">
        <div class={`${icon} text-xl text-brand-primary`}></div>
        <div class="flex flex-col items-start">
          <span class="text-white font-outfit font-extrabold text-sm tracking-tight group-hover:text-brand-primary transition-colors">
            {name}
          </span>
          <span class="text-[9px] text-gray-500 font-black uppercase tracking-widest mt-0.5">
            Resource Parser
          </span>
        </div>
      </div>

      <div class={`
        p-2 rounded-lg border transition-all duration-300
        ${copied 
          ? 'bg-brand-primary border-brand-primary text-white scale-110' 
          : 'bg-white/5 border-white/5 text-gray-500 group-hover:text-brand-primary group-hover:border-brand-primary/20 group-hover:bg-brand-primary/5'
        }
      `}>
        <div class={`${copied ? 'i-ph-check-bold' : 'i-ph-copy-bold'} text-xs`}></div>
      </div>

      {copied && (
        <div class="absolute -top-1 -right-1 px-1.5 py-0.5 bg-brand-primary text-white text-[7px] font-black uppercase tracking-widest rounded-md shadow-lg animate-bounce">
          Copied
        </div>
      )}
    </button>
  );
}
