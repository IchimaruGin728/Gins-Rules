import DynamicIcon from "./DynamicIcon.preact";

interface Props {
  icon: string;
  accent: string;
  active?: boolean;
  size?: "sm" | "md" | "lg";
  class?: string;
}

const sizeClasses = {
  sm: {
    frame: "w-8 h-8 rounded-2xl p-1",
    icon: "w-full h-full rounded-[0.8rem]",
  },
  md: {
    frame: "w-10 h-10 rounded-[1.15rem] p-1",
    icon: "w-full h-full rounded-[0.95rem]",
  },
  lg: {
    frame: "w-14 h-14 rounded-[1.4rem] p-1.5",
    icon: "w-full h-full rounded-[1.05rem]",
  },
} as const;

export default function AppLogo({
  icon,
  accent,
  active = false,
  size = "md",
  class: className = "",
}: Props) {
  const classes = sizeClasses[size];

  return (
    <div
      class={`
        app-logo-shell relative inline-flex shrink-0 items-center justify-center overflow-hidden
        border border-white/10 bg-white/[0.04] shadow-[0_18px_45px_rgba(0,0,0,0.28)]
        transition-all duration-300
        ${classes.frame}
        ${active ? "scale-105 border-white/18 bg-white/[0.07]" : "opacity-90"}
        ${className}
      `}
      style={{
        boxShadow: active
          ? `0 0 0 1px ${accent}26, 0 18px 45px rgba(0, 0, 0, 0.32)`
          : undefined,
      }}
    >
      <div
        class="pointer-events-none absolute inset-0 opacity-80"
        style={{
          background: active
            ? `radial-gradient(circle at top, ${accent}2e, transparent 58%)`
            : "radial-gradient(circle at top, rgba(255,255,255,0.08), transparent 58%)",
        }}
      ></div>
      <DynamicIcon
        icon={icon}
        class={`
          ${classes.icon}
          ${active ? "app-logo-image app-logo-image-active" : "app-logo-image app-logo-image-muted"}
        `}
      />
    </div>
  );
}
