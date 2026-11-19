import wasm from 'vite-plugin-wasm';

export default {
  root: 'sources/',
  publicDir: '../static/',
  envDir: false,
  base: './',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    license: { fileName: 'dependency-licenses.md' },
  },
  plugins: [wasm()],
};
