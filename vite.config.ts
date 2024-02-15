import { defineConfig } from "vite";
import { nodePolyfills } from 'vite-plugin-node-polyfills'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    nodePolyfills(),
  ],
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },

  build: {
    rollupOptions: {
      input: {
        index: "./index.html",
      },
    },
    chunkSizeWarningLimit: 100000000,
    outDir: "static",
    assetsDir: "",
  },
}));
