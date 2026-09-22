import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig(({ command, isPreview }) => ({
  root: import.meta.dirname,
  base: command === "build" || isPreview ? "/liquidfun-rs/" : "/",
  plugins: [tailwindcss(), solid()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
}));
