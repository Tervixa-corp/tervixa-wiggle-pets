import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The Tauri shell serves the built bundle from `dist`;
// `cargo tauri dev` points the devUrl here when live-reloading.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
});
