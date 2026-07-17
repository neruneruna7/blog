/**
 * なぜ実装するのか:
 * - Astroの最終成果物に、生成メタデータ・本文・OGP画像がそろっていることを検証するため。
 *
 * 期待する動作:
 * - 各記事のページと本文を確認し、記事JSONが参照するOGP画像がfrontend/distに存在しない場合は失敗する。
 *
 * 使い方の概要:
 * - frontendの `test` スクリプトから、Astroビルド直後に実行する。
 */
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";

const frontendRoot = new URL("../", import.meta.url);
const repoRoot = new URL("../../", import.meta.url);
const distRoot = new URL("dist/", frontendRoot);
const generatedRoot = new URL("generated/", repoRoot);

const site = await readJson(new URL("site.json", generatedRoot));
const articles = await readJson(new URL("article-index.json", generatedRoot));

assert(existsSync(new URL("index.html", distRoot)), "frontend/dist/index.html must exist.");

const indexHtml = await readFile(new URL("index.html", distRoot), "utf8");
assert(indexHtml.includes(site.title), "index.html must contain the site title.");

assert(Array.isArray(articles), "generated/article-index.json must be an array.");
assert(articles.length > 0, "generated/article-index.json must contain at least one article.");
assert(
  indexHtml.includes(articles[0].title),
  "index.html must contain at least one article title.",
);

for (const article of articles) {
  const articlePage = new URL(`articles/${article.slug}/index.html`, distRoot);
  assert(existsSync(articlePage), `${articlePage.pathname} must exist.`);

  const articleHtml = await readFile(articlePage, "utf8");
  const bodyHtml = await readFile(
    new URL(`articles/${article.slug}/body.html`, generatedRoot),
    "utf8",
  );
  const bodyText = extractText(bodyHtml);

  assert(articleHtml.includes(article.title), `${article.slug} page must contain article title.`);
  assert(
    bodyText.length === 0 || extractText(articleHtml).includes(bodyText),
    `${article.slug} page must contain article body content.`,
  );
  assert(
    !articleHtml.includes("client:load") &&
      !articleHtml.includes("client:idle") &&
      !articleHtml.includes("client:visible"),
    `${article.slug} page must not require Astro client hydration for article body.`,
  );
  assert(
    typeof article.ogpImage?.path === "string" &&
      article.ogpImage.path.startsWith("/ogp/articles/"),
    `${article.slug} must reference an OGP image under /ogp/articles/.`,
  );
  assert(
    !article.ogpImage.path.includes("\\"),
    `${article.slug} OGP image path must not contain OS-dependent separators.`,
  );
  assert(
    existsSync(new URL(article.ogpImage.path.slice(1), distRoot)),
    `${article.ogpImage.path} must exist in frontend/dist.`,
  );

  const articleUrl = new URL(article.url, site.baseUrl).href;
  const ogpImageUrl = new URL(article.ogpImage.path, site.baseUrl).href;
  assertMeta(articleHtml, "property", "og:type", "article", article.slug);
  assertMeta(articleHtml, "property", "og:title", article.title, article.slug);
  assertMeta(articleHtml, "property", "og:url", articleUrl, article.slug);
  assertMeta(articleHtml, "property", "og:image", ogpImageUrl, article.slug);
  assertMeta(
    articleHtml,
    "property",
    "og:image:width",
    String(article.ogpImage.width),
    article.slug,
  );
  assertMeta(
    articleHtml,
    "property",
    "og:image:height",
    String(article.ogpImage.height),
    article.slug,
  );
}

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function extractText(html) {
  return html.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function assertMeta(html, attribute, key, content, slug) {
  const escapedKey = escapeRegExp(escapeHtmlAttribute(key));
  const escapedContent = escapeRegExp(escapeHtmlAttribute(content));
  const metaPattern = new RegExp(
    `<meta\\s+${attribute}="${escapedKey}"\\s+content="${escapedContent}"\\s*\\/?>`,
  );
  assert(metaPattern.test(html), `${slug} must contain ${key}=${content}.`);
}

function escapeHtmlAttribute(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
