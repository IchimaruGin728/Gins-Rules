import type { CSSProperties } from "preact/compat";

interface Props {
  icon: string;
  class?: string;
  style?: CSSProperties;
}

export default function DynamicIcon({
  icon,
  class: className = "",
  style = {},
}: Props) {
  const isImage =
    icon.startsWith("/") || icon.startsWith("http") || icon.includes(".");

  if (isImage) {
    return (
      <img
        src={icon}
        class={`${className} object-contain`}
        style={style}
        alt=""
      />
    );
  }

  return <div class={`${icon} ${className}`} style={style}></div>;
}
