import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    outDir: 'static',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        style: resolve(__dirname, 'css/style.css'),
        themeconfig: resolve(__dirname, 'tsrc/themeconfig.ts'),
        overlay: resolve(__dirname, 'tsrc/overlay.ts'),
        // Add more entry points as needed
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name].js',
        assetFileNames: '[name].[ext]'
      }
    }
  }
});