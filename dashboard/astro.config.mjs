import cloudflare from '@astrojs/cloudflare';
import preact from '@astrojs/preact';
import { defineConfig } from 'astro/config';
import UnoCSS from 'unocss/astro';

export default defineConfig({
  output: 'server',
  adapter: cloudflare({
    imageService: 'passthrough',
  }),
  integrations: [
    preact(),
    UnoCSS({
      injectReset: true,
    }),
  ],
});
