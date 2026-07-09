import { defineConfig } from "astro/config";

export default defineConfig({
  output: "static",
  server: {
    host: true,
    port: 3000,
    allowedHosts: true,
    },
});
