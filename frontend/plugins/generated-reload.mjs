import { createHash } from "node:crypto";
import { readdir, readFile } from "node:fs/promises";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const DEFAULT_DEBOUNCE_MS = 300;
const DEFAULT_GENERATED_ROOT = fileURLToPath(new URL("../../generated", import.meta.url));
const ARTICLE_INDEX_PATH = "article-index.json";

/**
 * 生成物の変更だけを拾うVite開発サーバープラグイン。
 * 内容が同じファイル書き換えを無視し、slug集合が変わった場合のみサーバーを再起動する。
 */
export default function generatedReload({
  generatedRoot = DEFAULT_GENERATED_ROOT,
  debounceMs = DEFAULT_DEBOUNCE_MS,
} = {}) {
  const absoluteGeneratedRoot = resolve(generatedRoot);

  return {
    name: "generated-reload",
    apply: "serve",

    async configureServer(server) {
      let disposed = false;
      let initialized = false;
      let pendingInitialEvent = false;
      let timer;
      let reconciling = false;
      let reconcileAgain = false;
      let previousSnapshot = emptySnapshot();

      const cleanup = () => {
        if (disposed) return;
        disposed = true;
        clearTimeout(timer);
        server.watcher.off("all", onFileEvent);
        server.watcher.off("close", cleanup);
      };

      const logFailure = (error) => {
        server.config.logger.warn(
          `[generated-reload] ${error instanceof Error ? error.message : String(error)}`,
        );
      };

      const reconcile = async () => {
        timer = undefined;
        if (disposed) return;
        if (reconciling) {
          reconcileAgain = true;
          return;
        }

        reconciling = true;
        try {
          const nextSnapshot = await readGeneratedSnapshot(absoluteGeneratedRoot);
          if (disposed) return;

          const hasContentChanged = !sameContent(previousSnapshot.files, nextSnapshot.files);
          const hasRouteChanged = !sameSlugs(previousSnapshot.slugs, nextSnapshot.slugs);
          previousSnapshot = nextSnapshot;

          if (hasRouteChanged) {
            await server.restart();
          } else if (hasContentChanged) {
            server.ws.send({ type: "full-reload" });
          }
        } catch (error) {
          logFailure(error);
        } finally {
          reconciling = false;
          if (reconcileAgain && !disposed) {
            reconcileAgain = false;
            scheduleReconcile();
          }
        }
      };

      const scheduleReconcile = () => {
        if (disposed) return;
        clearTimeout(timer);
        timer = setTimeout(() => void reconcile(), debounceMs);
      };

      function onFileEvent(_event, filePath) {
        if (!isRelevantGeneratedEvent(absoluteGeneratedRoot, filePath)) return;
        if (!initialized) {
          pendingInitialEvent = true;
          return;
        }
        scheduleReconcile();
      }

      // Vite owns this watcher, so the per-server listener and timer end with it on restart.
      server.watcher.on("all", onFileEvent);
      server.watcher.on("close", cleanup);
      server.watcher.add(absoluteGeneratedRoot);

      try {
        previousSnapshot = await readGeneratedSnapshot(absoluteGeneratedRoot);
      } catch (error) {
        logFailure(error);
      }

      initialized = true;
      if (pendingInitialEvent) scheduleReconcile();
    },
  };
}

/**
 * ファイルイベントの対象を、ブラウザ表示へ影響する生成物のみに絞る。
 */
export function isRelevantGeneratedEvent(generatedRoot, filePath) {
  const absoluteGeneratedRoot = resolve(generatedRoot);
  const absoluteFilePath = isAbsolute(filePath)
    ? resolve(filePath)
    : resolve(absoluteGeneratedRoot, filePath);
  const relativePath = relative(absoluteGeneratedRoot, absoluteFilePath);

  if (relativePath === "") return true;
  if (relativePath === ".." || relativePath.startsWith(`..${sep}`)) return false;

  const normalizedPath = relativePath.split(sep).join("/");
  const segments = normalizedPath.split("/");
  if (
    segments.some((segment) => segment === ".cache") ||
    normalizedPath === ".frontend-public-manifest.json" ||
    normalizedPath.endsWith(".tmp")
  ) {
    return false;
  }

  if (normalizedPath === ARTICLE_INDEX_PATH || normalizedPath === "site.json") {
    return true;
  }
  if (normalizedPath === "articles" || normalizedPath.startsWith("articles/")) {
    return true;
  }
  return ["public-root/ogp", "public-root/ogp/articles"].some(
    (prefix) => normalizedPath === prefix || normalizedPath.startsWith(`${prefix}/`),
  );
}

/**
 * 表示対象ファイルのハッシュと記事slug集合を一度の走査で取得する。
 */
export async function readGeneratedSnapshot(generatedRoot) {
  const absoluteGeneratedRoot = resolve(generatedRoot);
  const paths = await collectGeneratedFiles(absoluteGeneratedRoot);
  const fileEntries = await Promise.all(
    paths.map(async (path) => {
      try {
        const contents = await readFile(path);
        return [path, createHash("sha256").update(contents).digest("hex"), contents];
      } catch (error) {
        if (error?.code === "ENOENT") return undefined;
        throw error;
      }
    }),
  );

  const files = new Map();
  let articleIndexContents;
  for (const entry of fileEntries) {
    if (!entry) continue;
    const [path, digest, contents] = entry;
    files.set(path, digest);
    if (path === resolve(absoluteGeneratedRoot, ARTICLE_INDEX_PATH)) {
      articleIndexContents = contents;
    }
  }

  return {
    files,
    slugs: articleIndexContents ? parseArticleSlugs(articleIndexContents) : null,
  };
}

async function collectGeneratedFiles(generatedRoot) {
  const rootEntries = await readDirectoryOrEmpty(generatedRoot);
  const paths = [];

  for (const entry of rootEntries) {
    const path = resolve(generatedRoot, entry.name);
    if (entry.isFile() && [ARTICLE_INDEX_PATH, "site.json"].includes(entry.name)) {
      paths.push(path);
    } else if (entry.isDirectory() && entry.name === "articles") {
      paths.push(...(await collectArticleFiles(path)));
    } else if (entry.isDirectory() && entry.name === "public-root") {
      paths.push(...(await collectOgpFiles(resolve(path, "ogp", "articles"))));
    }
  }

  return paths;
}

async function collectArticleFiles(directory) {
  const entries = await readDirectoryOrEmpty(directory);
  const paths = [];

  for (const entry of entries) {
    if (entry.name === ".cache" || entry.name.endsWith(".tmp")) continue;

    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      paths.push(...(await collectArticleFiles(path)));
    } else if (entry.isFile() && ["body.html", "meta.json"].includes(entry.name)) {
      paths.push(path);
    }
  }

  return paths;
}

async function collectOgpFiles(directory) {
  const entries = await readDirectoryOrEmpty(directory);
  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".png"))
    .map((entry) => resolve(directory, entry.name));
}

async function readDirectoryOrEmpty(path) {
  try {
    return await readdir(path, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT" || error?.code === "ENOTDIR") return [];
    throw error;
  }
}

function parseArticleSlugs(contents) {
  const articles = JSON.parse(contents.toString("utf8"));
  if (!Array.isArray(articles)) {
    throw new Error("generated/article-index.json must be an array.");
  }

  const slugs = articles.map((article) => article?.slug);
  if (slugs.some((slug) => typeof slug !== "string")) {
    throw new Error("generated/article-index.json entries must contain a string slug.");
  }
  return new Set(slugs);
}

function emptySnapshot() {
  return { files: new Map(), slugs: null };
}

function sameContent(left, right) {
  if (left.size !== right.size) return false;
  for (const [path, digest] of left) {
    if (right.get(path) !== digest) return false;
  }
  return true;
}

function sameSlugs(left, right) {
  if (left === null || right === null) return left === right;
  if (left.size !== right.size) return false;
  for (const slug of left) {
    if (!right.has(slug)) return false;
  }
  return true;
}
