import {
  defineConfig,
  presetWind,
  presetIcons,
  presetTypography,
} from 'unocss';
import transformerDirectives from '@unocss/transformer-directives';
import transformerVariantGroup from '@unocss/transformer-variant-group';

export default defineConfig({
  presets: [
    presetWind(),
    presetTypography(),
    presetIcons({
      scale: 1.2,
      warn: true,
      extraProperties: {
				'display': 'inline-block',
				'vertical-align': 'middle',
			},
    }),
  ],
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
  theme: {
    colors: {
      brand: {
        primary: '#C084FC',
        secondary: '#581C87',
      },
      navy: {
        900: '#050505',
      }
    },
    fontFamily: {
      outfit: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Outfit Variable", "Outfit", sans-serif',
      inter: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Inter Variable", "Inter", sans-serif',
    }
  },
  shortcuts: {
    // Optimized for Safari: explicit -webkit prefixes for backdrop-filter
    'glass-panel': 'bg-black/20 backdrop-blur-md border border-white/10 rounded-3xl p-6 shadow-2xl hover:border-white/20 transition-all duration-500',
    'nav-pill': 'bg-black/20 backdrop-blur-lg border border-white/10 rounded-full px-6 py-2 flex items-center gap-6 shadow-xl',
    'btn-brand': 'bg-gradient-to-r from-[#C084FC] to-[#581C87] text-white px-6 py-2 rounded-full font-bold hover:scale-105 transition-all duration-500 active:scale-95 shadow-lg shadow-brand-primary/20',
    'text-gradient': 'bg-clip-text text-transparent bg-gradient-to-r from-[#C084FC] to-[#581C87]',
  },
});
