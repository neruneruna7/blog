import { defineConfig } from "astro/config";
import react from '@astrojs/react';
import tailwindcss from "@tailwindcss/vite";

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
  publicDir: "../generated/public-root",
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
