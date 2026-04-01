import { signal } from "@preact/signals";

export type AppType =
  | "singbox"
  | "mihomo"
  | "stash"
  | "surge"
  | "quanx"
  | "loon"
  | "egern"
  | "shadowrocket"
  | "surfboard"
  | "exclave"
  | "v2ray";

export interface AppConfig {
  id: AppType;
  label: string;
  icon: string;
  ext: string;
  color: string;
}

export const APPS: AppConfig[] = [
  {
    id: "singbox",
    label: "Sing-box",
    icon: "/icons/singbox.jpg",
    ext: "srs",
    color: "#5D5CDE",
  },
  {
    id: "mihomo",
    label: "Mihomo",
    icon: "/icons/clashmi.jpg",
    ext: "mrs",
    color: "#00BFA5",
  },
  {
    id: "stash",
    label: "Stash",
    icon: "/icons/stash.jpg",
    ext: "mrs",
    color: "#4CAF50",
  },
  {
    id: "surge",
    label: "Surge",
    icon: "/icons/surge.jpg",
    ext: "list",
    color: "#FBC02D",
  },
  {
    id: "quanx",
    label: "QuanX",
    icon: "/icons/quanx.jpg",
    ext: "list",
    color: "#f44336",
  },
  {
    id: "loon",
    label: "Loon",
    icon: "/icons/loon.jpg",
    ext: "list",
    color: "#03A9F4",
  },
  {
    id: "egern",
    label: "Egern",
    icon: "/icons/egern.jpg",
    ext: "yaml",
    color: "#9C27B0",
  },
  {
    id: "shadowrocket",
    label: "Shadowrocket",
    icon: "/icons/shadowrocket.jpg",
    ext: "list",
    color: "#FF4081",
  },
  {
    id: "surfboard",
    label: "Surfboard",
    icon: "/icons/surfboard.png",
    ext: "list",
    color: "#68BBE3",
  },
  {
    id: "exclave",
    label: "Exclave",
    icon: "/icons/exclave.png",
    ext: "list",
    color: "#E91E63",
  },
  {
    id: "v2ray",
    label: "V2Ray / Xray",
    icon: "/icons/xray.png",
    ext: "dat",
    color: "#CFD8DC",
  },
];

export const activeApp = signal<AppType>("singbox");
export const optimizedDomainSet = signal<boolean>(false);
export const getActiveConfig = () => {
  const config = APPS.find((a) => a.id === activeApp.value);

  if (!config) {
    throw new Error(`Unknown app config: ${activeApp.value}`);
  }

  return config;
};
