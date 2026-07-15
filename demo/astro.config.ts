import { defineConfig } from 'astro/config'

export default defineConfig({
  site: 'https://laride.github.io',
  base: '/img2lv',
  markdown: {
    shikiConfig: {
      themes: {
        light: 'github-light',
        dark: 'github-dark',
      },
      defaultColor: false,
    },
  },
  vite: {
    define: { global: 'globalThis' },
    resolve: { alias: { buffer: 'buffer' } },
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
})
