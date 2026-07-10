import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";

/**
 * Astroはfrontendディレクトリをカレントディレクトリとして実行される.
 *
 * frontendの1階層上にあるgeneratedを，絶対パスとして解決する.
 */
const generatedRoot = resolve(process.cwd(), "..", "generated");

/**
 * generated配下のファイルパスを構築する.
 */
function generatedPath(...segments: string[]): string {
  return join(generatedRoot, ...segments);
}

/**
 * なぜ実装するのか:
 * - Rustエンジンが生成したサイト設定JSONをAstroで型付きに扱うため.
 *
 * 期待する動作:
 * - `generated/site.json` の契約を表す.
 *
 * 使い方の概要:
 * - `getSiteConfig` の戻り値としてレイアウトやページの表示に使う.
 */
export type SiteConfig = {
  title: string;
  description: string;
  language: string;
  baseUrl: string;
};

/**
 * なぜ実装するのか:
 * - Rustエンジンが生成した記事メタデータJSONをAstroで型付きに扱うため.
 *
 * 期待する動作:
 * - `generated/article-index.json` と `generated/articles/<slug>/meta.json` の契約を表す.
 *
 * 使い方の概要:
 * - 記事一覧、静的パス生成、記事ページの表示に使う.
 */
export type ArticleMeta = {
  id: string;
  slug: string;
  title: string;
  description: string;
  publishedAt: string;
  updatedAt: string | null;
  tags: string[];
  url: string;
  ogpImage: string | null;
};

/**
 * なぜ実装するのか:
 * - Astroビルド時にRust生成済みサイト設定を読み込むため.
 *
 * 期待する動作:
 * - `generated/site.json` を読み、サイト設定として返す.
 *
 * 使い方の概要:
 * - ページやレイアウトのfrontmatterでawaitして利用する.
 */
export async function getSiteConfig(): Promise<SiteConfig> {
  const value = await readJson(generatedPath("site.json"));
  return asSiteConfig(value, "generated/site.json");
}

/**
 * なぜ実装するのか:
 * - Astroビルド時にRust生成済み記事一覧を読み込むため.
 *
 * 期待する動作:
 * - `generated/article-index.json` を読み、記事メタデータ配列として返す.
 *
 * 使い方の概要:
 * - トップページの記事カード一覧と `getStaticPaths` で利用する.
 */
export async function getArticles(): Promise<ArticleMeta[]> {
  const value = await readJson(generatedPath("article-index.json"));

  if (!Array.isArray(value)) {
    throw new Error("generated/article-index.json must be an array.");
  }

  return value.map((item, index) =>
    asArticleMeta(item, `generated/article-index.json[${index}]`),
  );
}

/**
 * なぜ実装するのか:
 * - 記事ページが記事別メタデータの生成物を直接参照できるようにするため.
 *
 * 期待する動作:
 * - `generated/articles/<slug>/meta.json` を読み、記事メタデータとして返す.
 *
 * 使い方の概要:
 * - 記事詳細ページの静的生成時にslugからメタデータを取得する.
 */
export async function getArticleMeta(slug: string): Promise<ArticleMeta> {
  const value = await readJson(
    generatedPath("articles", slug, "meta.json"),
  );

  return asArticleMeta(value, `generated/articles/${slug}/meta.json`);
}

 /**
 * なぜ実装するのか:
 * - Astroビルド時にRust生成済みの本文HTML断片を読み込むため.
 *
 * 期待する動作:
 * - `generated/articles/<slug>/body.html` の文字列を返す.
 *
 * 使い方の概要:
 * - 記事ページで `set:html` に渡して静的HTMLへ埋め込む.
 */
export async function getArticleHtml(slug: string): Promise<string> {
  return readFile(
    generatedPath("articles", slug, "body.html"),
    "utf8",
  );
}

async function readJson(path: string): Promise<unknown> {
  return JSON.parse(await readFile(path, "utf8")) as unknown;
}
function asSiteConfig(value: unknown, source: string): SiteConfig {
  if (!isRecord(value)) {
    throw new Error(`${source} must be an object.`);
  }

  assertString(value.title, `${source}.title`);
  assertString(value.description, `${source}.description`);
  assertString(value.language, `${source}.language`);
  assertString(value.baseUrl, `${source}.baseUrl`);

  return {
    title: value.title,
    description: value.description,
    language: value.language,
    baseUrl: value.baseUrl,
  };
}

function asArticleMeta(value: unknown, source: string): ArticleMeta {
  if (!isRecord(value)) {
    throw new Error(`${source} must be an object.`);
  }

  assertString(value.id, `${source}.id`);
  assertString(value.slug, `${source}.slug`);
  assertString(value.title, `${source}.title`);
  assertString(value.description, `${source}.description`);
  assertString(value.publishedAt, `${source}.publishedAt`);
  assertStringOrNull(value.updatedAt, `${source}.updatedAt`);
  assertStringArray(value.tags, `${source}.tags`);
  assertString(value.url, `${source}.url`);
  assertStringOrNull(value.ogpImage, `${source}.ogpImage`);

  return {
    id: value.id,
    slug: value.slug,
    title: value.title,
    description: value.description,
    publishedAt: value.publishedAt,
    updatedAt: value.updatedAt,
    tags: value.tags,
    url: value.url,
    ogpImage: value.ogpImage,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertString(value: unknown, field: string): asserts value is string {
  if (typeof value !== "string") {
    throw new Error(`${field} must be a string.`);
  }
}

function assertStringOrNull(value: unknown, field: string): asserts value is string | null {
  if (typeof value !== "string" && value !== null) {
    throw new Error(`${field} must be a string or null.`);
  }
}

function assertStringArray(value: unknown, field: string): asserts value is string[] {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new Error(`${field} must be an array of strings.`);
  }
}
