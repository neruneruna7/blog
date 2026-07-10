import { defineConfig } from "astro/config";
import react from '@astrojs/react';
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  output: "static",
  server: {
    host: true,
    port: 3000,
    allowedHosts: true,
  },
  integrations: [react()],
  vite: {
      plugins: [tailwindcss()],
    },
});
