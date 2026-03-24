import { signal } from "@preact/signals";

export type AppType = 
  | 'singbox' 
  | 'mihomo' 
  | 'stash' 
  | 'surge' 
  | 'quanx' 
  | 'loon' 
  | 'egern' 
  | 'shadowrocket'
  | 'surfboard';

export interface AppConfig {
  id: AppType;
  label: string;
  icon: string;
  ext: string;
  color: string;
}

export const APPS: AppConfig[] = [
  { id: 'singbox', label: 'Sing-box', icon: 'i-ph-cube-fill', ext: 'srs', color: '#5D5CDE' },
  { id: 'mihomo', label: 'Mihomo', icon: '/mihomo.png', ext: 'mrs', color: '#00BFA5' },
  { id: 'stash', label: 'Stash', icon: 'i-ph-leaf-fill', ext: 'mrs', color: '#4CAF50' },
  { id: 'surge', label: 'Surge', icon: 'i-ph-lightning-fill', ext: 'list', color: '#FBC02D' },
  { id: 'quanx', label: 'QuanX', icon: 'i-ph-shield-check-fill', ext: 'list', color: '#f44336' },
  { id: 'loon', label: 'Loon', icon: 'i-ph-balloon-fill', ext: 'list', color: '#03A9F4' },
  { id: 'egern', label: 'Egern', icon: 'i-ph-circles-three-plus-fill', ext: 'yaml', color: '#9C27B0' },
  { id: 'shadowrocket', label: 'Shadowrocket', icon: 'i-ph-rocket-fill', ext: 'list', color: '#FF4081' },
  { id: 'surfboard', label: 'Surfboard', icon: 'i-ph-surfboard-fill', ext: 'list', color: '#68BBE3' },
];

export const activeApp = signal<AppType>('singbox');
export const getActiveConfig = () => APPS.find(a => a.id === activeApp.value)!;
