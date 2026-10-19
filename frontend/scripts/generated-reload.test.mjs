import { strict as assert } from "node:assert";
import { EventEmitter } from "node:events";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";

import generatedReload, {
  isRelevantGeneratedEvent,
  readGeneratedSnapshot,
} from "../plugins/generated-reload.mjs";

class FakeWatcher extends EventEmitter {
  addedPaths = [];

  add(path) {
    this.addedPaths.push(path);
  }
}

async function createFixture() {
  const root = await mkdtemp(join(tmpdir(), "generated-reload-"));
  return { root, generatedRoot: join(root, "generated") };
}

async function writeGeneratedFile(generatedRoot, path, contents) {
  const filePath = join(generatedRoot, path);
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, contents);
  return filePath;
}

function createFakeServer() {
  const watcher = new FakeWatcher();
  const messages = [];
  const warnings = [];
  const server = {
    watcher,
    ws: { send: (message) => messages.push(message) },
    config: { logger: { warn: (message) => warnings.push(message) } },
    restartCount: 0,
    async restart() {
      this.restartCount += 1;
    },
  };

  return { server, watcher, messages, warnings };
}

async function startPlugin(generatedRoot, debounceMs = 15) {
  const fake = createFakeServer();
  const plugin = generatedReload({ generatedRoot, debounceMs });
  await plugin.configureServer(fake.server);
  return { ...fake, plugin };
}

function emitGeneratedEvent(watcher, generatedRoot, event, path) {
  watcher.emit("all", event, join(generatedRoot, ...path.split("/")));
}

async function waitFor(predicate, message, timeoutMs = 1000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
  assert.fail(message);
}

async function createArticle(generatedRoot, slug, title, body) {
  const article = {
    slug,
    title,
    description: `${title} description`,
    publishedAt: "2026-01-01",
    updatedAt: null,
    tags: [],
    url: `/articles/${slug}/`,
    ogpImage: {
      path: `/ogp/articles/${slug}.png`,
      width: 1200,
      height: 630,
      mimeType: "image/png",
      alt: `${title} | Example Blog`,
    },
  };
  await writeGeneratedFile(generatedRoot, `articles/${slug}/meta.json`, JSON.stringify(article));
  await writeGeneratedFile(generatedRoot, `articles/${slug}/body.html`, body);
  await writeGeneratedFile(
    generatedRoot,
    `public-root/ogp/articles/${slug}.png`,
    Buffer.from([1, 2, 3]),
  );
  return article;
}

async function writeArticleIndex(generatedRoot, articles) {
  await writeGeneratedFile(
    generatedRoot,
    "article-index.json",
    `${JSON.stringify(articles)}\n`,
  );
}

test("生成物の表示対象と除外対象を判別する", () => {
  const generatedRoot = resolve("/tmp/generated");

  for (const path of [
    "articles/example/body.html",
    "articles/example/meta.json",
    "article-index.json",
    "site.json",
    "public-root/ogp/articles/example.png",
  ]) {
    assert.equal(isRelevantGeneratedEvent(generatedRoot, join(generatedRoot, path)), true);
  }

  for (const path of [
    ".cache/manifest.json",
    ".frontend-public-manifest.json",
    "articles/example/body.html.123.tmp",
    "public-root/favicon.svg",
  ]) {
    assert.equal(isRelevantGeneratedEvent(generatedRoot, join(generatedRoot, path)), false);
  }
  assert.equal(isRelevantGeneratedEvent(generatedRoot, "/tmp/elsewhere/site.json"), false);
});

test("generatedが存在しない初期状態を空として読み込む", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));

  const snapshot = await readGeneratedSnapshot(fixture.generatedRoot);

  assert.deepEqual([...snapshot.files], []);
  assert.equal(snapshot.slugs, null);
});

test("監視開始後に生成ディレクトリが作成されるとルートを再構築する", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));
  const { server, watcher, messages } = await startPlugin(fixture.generatedRoot);

  assert.deepEqual(watcher.addedPaths, [resolve(fixture.generatedRoot)]);
  const article = await createArticle(
    fixture.generatedRoot,
    "first",
    "First article",
    "<p>first body</p>",
  );
  await writeGeneratedFile(fixture.generatedRoot, "site.json", JSON.stringify({
    title: "Example Blog",
    description: "Example",
    language: "en",
  }));
  await writeArticleIndex(fixture.generatedRoot, [article]);
  emitGeneratedEvent(watcher, fixture.generatedRoot, "addDir", "articles/first");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "add", "article-index.json");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "add", "site.json");

  await waitFor(() => server.restartCount === 1, "記事一覧作成後のサーバー再起動がない");
  assert.deepEqual(messages, []);
});

test("同じ内容の書き換えとキャッシュ・Manifest・一時ファイルでは通知しない", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));
  const article = await createArticle(
    fixture.generatedRoot,
    "alpha",
    "Alpha",
    "<p>same</p>",
  );
  await writeArticleIndex(fixture.generatedRoot, [article]);
  const { server, watcher, messages } = await startPlugin(fixture.generatedRoot);

  await writeGeneratedFile(fixture.generatedRoot, "articles/alpha/body.html", "<p>same</p>");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "articles/alpha/body.html");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", ".cache/manifest.json");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", ".frontend-public-manifest.json");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "add", "site.json.random.tmp");
  await new Promise((resolve) => setTimeout(resolve, 60));

  assert.deepEqual(messages, []);
  assert.equal(server.restartCount, 0);
});

test("複数の本文・サイト情報・OGP変更を一度の全体リロードへ集約する", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));
  const article = await createArticle(
    fixture.generatedRoot,
    "alpha",
    "Alpha",
    "<p>before</p>",
  );
  await writeArticleIndex(fixture.generatedRoot, [article]);
  await writeGeneratedFile(fixture.generatedRoot, "site.json", "{\"title\":\"Before\"}");
  const { server, watcher, messages } = await startPlugin(fixture.generatedRoot);

  await writeGeneratedFile(fixture.generatedRoot, "articles/alpha/body.html", "<p>after</p>");
  await writeGeneratedFile(fixture.generatedRoot, "site.json", "{\"title\":\"After\"}");
  await writeGeneratedFile(
    fixture.generatedRoot,
    "public-root/ogp/articles/alpha.png",
    Buffer.from([4, 5, 6]),
  );
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "articles/alpha/body.html");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "site.json");
  emitGeneratedEvent(
    watcher,
    fixture.generatedRoot,
    "change",
    "public-root/ogp/articles/alpha.png",
  );

  await waitFor(() => messages.length === 1, "生成物の変更がリロード通知されない");
  await new Promise((resolve) => setTimeout(resolve, 60));
  assert.deepEqual(messages, [{ type: "full-reload" }]);
  assert.equal(server.restartCount, 0);
});

test("記事slug追加・削除時だけサーバーを再起動する", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));
  const first = await createArticle(fixture.generatedRoot, "first", "First", "<p>first</p>");
  await writeArticleIndex(fixture.generatedRoot, [first]);
  const { server, watcher, messages } = await startPlugin(fixture.generatedRoot);

  const second = await createArticle(fixture.generatedRoot, "second", "Second", "<p>second</p>");
  await writeArticleIndex(fixture.generatedRoot, [first, second]);
  emitGeneratedEvent(watcher, fixture.generatedRoot, "add", "article-index.json");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "add", "articles/second/meta.json");
  await waitFor(() => server.restartCount === 1, "新規slugでサーバーが再起動しない");

  await writeArticleIndex(fixture.generatedRoot, [second, { ...first, title: "Renamed" }]);
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "article-index.json");
  await waitFor(() => messages.length === 1, "タイトル変更で全体リロードしない");
  assert.equal(server.restartCount, 1);

  await writeArticleIndex(fixture.generatedRoot, [second]);
  emitGeneratedEvent(watcher, fixture.generatedRoot, "unlink", "articles/first/meta.json");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "article-index.json");
  await waitFor(() => server.restartCount === 2, "slug削除でサーバーが再起動しない");
  assert.deepEqual(messages, [{ type: "full-reload" }]);
});

test("サーバー再起動時に旧リスナーを破棄し、新しい監視を一度だけ登録する", async (context) => {
  const fixture = await createFixture();
  context.after(() => rm(fixture.root, { recursive: true, force: true }));
  const article = await createArticle(fixture.generatedRoot, "alpha", "Alpha", "<p>before</p>");
  await writeArticleIndex(fixture.generatedRoot, [article]);
  const { server, watcher, plugin } = await startPlugin(fixture.generatedRoot, 40);

  await writeGeneratedFile(fixture.generatedRoot, "articles/alpha/body.html", "<p>after</p>");
  emitGeneratedEvent(watcher, fixture.generatedRoot, "change", "articles/alpha/body.html");
  watcher.emit("close");
  await new Promise((resolve) => setTimeout(resolve, 70));

  assert.equal(watcher.listenerCount("all"), 0);
  assert.equal(server.restartCount, 0);

  const next = createFakeServer();
  await plugin.configureServer(next.server);
  await writeGeneratedFile(fixture.generatedRoot, "articles/alpha/body.html", "<p>latest</p>");
  emitGeneratedEvent(next.watcher, fixture.generatedRoot, "change", "articles/alpha/body.html");
  await waitFor(() => next.messages.length === 1, "再起動後に変更を検出できない");

  assert.equal(next.watcher.listenerCount("all"), 1);
  assert.deepEqual(next.messages, [{ type: "full-reload" }]);
});
