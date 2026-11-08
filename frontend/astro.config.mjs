import { defineConfig } from "astro/config";
import { existsSync } from "node:fs";
import { loadEnvFile } from "node:process";
import { fileURLToPath } from "node:url";
import react from "@astrojs/react";
import tailwindcss from "@tailwindcss/vite";
import generatedReload from "./plugins/generated-reload.mjs";

// Astro configでは.envが自動読込されず、対象ファイルもfrontendの外にあるため明示的に読む。
const envFile = fileURLToPath(new URL("../.env", import.meta.url));
if (existsSync(envFile)) {
  loadEnvFile(envFile);
}

const siteUrl = resolveSiteUrl(process.env.BASE_URL);

/**
 * なぜ実装するのか:
 * - Rust生成資産とfrontend管理資産を統合した公開ルートをAstroへ渡すため。
 *
 * 期待する動作:
 * - `generated/public-root` の資産を開発サーバーと静的ビルドの公開ルートとして扱う。
 *
 * 使い方の概要:
 * - `assemble-public` を先に実行してからAstroを起動またはビルドする。
 */
export default defineConfig({
  output: "static",
  site: siteUrl,
  publicDir: "../generated/public-root",
  server: {
    host: true,
    port: 3000,
    allowedHosts: true,
  },
  integrations: [react()],
  vite: {
    plugins: [tailwindcss(), generatedReload()],
  },
});

function resolveSiteUrl(value) {
  if (!value?.trim()) {
    throw new Error("BASE_URL must be set in the repository .env or build environment.");
  }

  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("BASE_URL must be an absolute HTTP(S) origin, such as https://example.com.");
  }

  if (
    !["http:", "https:"].includes(url.protocol) ||
    url.pathname !== "/" ||
    url.search ||
    url.hash ||
    url.username ||
    url.password
  ) {
    throw new Error("BASE_URL must be an HTTP(S) origin without a path, query, or credentials.");
  }

  return url.origin;
}
