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
}

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function extractText(html) {
  return html.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
