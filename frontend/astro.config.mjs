import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  output: "static",
  server: {
    host: true,
    port: 3000,
    allowedHosts: true,
    },
  vite: {
      plugins: [tailwindcss()],
    },
});
