import type { ComponentChildren } from 'preact';

interface Props {
  children?: ComponentChildren;
  title?: string;
  activePath?: string;
}

export default function Layout({ children, title, activePath }: Props) {
  const pageTitle = title ? `${title} | Gins-Rules` : 'Gins-Rules Dashboard';

  return (
    <div class="min-h-screen" data-title={pageTitle}>
      <div class="bg-glow" />

      <header class="fixed top-6 left-0 right-0 z-50 flex justify-center px-4">
        <nav class="nav-pill">
          <div class="flex items-center gap-2 mr-4">
            <div class="i-ph-shield-check-fill text-2xl text-brand-primary" />
            <span class="text-xl font-extrabold font-outfit text-gradient tracking-tight">
              Gins
            </span>
          </div>

          <div class="flex items-center gap-6 text-sm font-medium text-gray-400">
            <a
              href="/"
              class={`hover:text-white transition-colors duration-300 flex items-center gap-1.5 ${activePath === "/" ? "text-white font-bold" : ""}`}
            >
              <div class="i-ph-house-bold" /> Home
            </a>
            <a
              href="/search"
              class={`hover:text-white transition-colors duration-300 flex items-center gap-1.5 ${activePath === "/search" ? "text-white font-bold" : ""}`}
            >
              <div class="i-ph-magnifying-glass-bold" /> Search
            </a>
            <a
              href="/ai"
              class={`hover:text-white transition-colors duration-300 flex items-center gap-1.5 ${activePath === "/ai" ? "text-white font-bold" : ""}`}
            >
              <div class="i-ph-sparkle-bold" /> AI Monitor
            </a>
            <a
              href="/status"
              class={`hover:text-white transition-colors duration-300 flex items-center gap-1.5 ${activePath === "/status" ? "text-white font-bold" : ""}`}
            >
              <div class="i-ph-pulse-bold" /> Status
            </a>
          </div>
        </nav>
      </header>

      <main class="max-w-7xl mx-auto pt-32 p-6 md:p-8 animate-fade-in-up">{children}</main>

      <footer class="mt-20 py-12 border-t border-white/5 text-center text-gray-600 text-sm font-medium tracking-wide">
        <p>&copy; 2026 GINS-RULES. POWERED BY CLOUDFLARE EDGE.</p>
      </footer>
    </div>
  );
}
