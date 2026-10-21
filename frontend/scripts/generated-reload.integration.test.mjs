import { strict as assert } from "node:assert";
import { spawn } from "node:child_process";
import { cp, mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import net from "node:net";
import { tmpdir } from "node:os";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const frontendRoot = fileURLToPath(new URL("../", import.meta.url));
const astroCli = join(frontendRoot, "node_modules", "astro", "bin", "astro.mjs");

async function createFixture() {
  const root = await mkdtemp(join(tmpdir(), "astro-generated-reload-"));
  const frontend = join(root, "frontend");
  const generated = join(root, "generated");

  await cp(frontendRoot, frontend, {
    recursive: true,
    filter(path) {
      const relativePath = relative(frontendRoot, path);
      return !relativePath.split(sep).some((segment) =>
        ["node_modules", ".astro", "dist"].includes(segment),
      );
    },
  });
  await symlink(join(frontendRoot, "node_modules"), join(frontend, "node_modules"), "dir");

  return { root, frontend, generated };
}

async function writeGeneratedFile(generatedRoot, path, contents) {
  const filePath = join(generatedRoot, path);
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, contents);
}

function createArticle(slug, title, body) {
  return {
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
    body,
  };
}

async function writeArticle(generatedRoot, article, imageBytes) {
  const { body, ...metadata } = article;
  await writeGeneratedFile(
    generatedRoot,
    `articles/${article.slug}/meta.json`,
    `${JSON.stringify(metadata)}\n`,
  );
  await writeGeneratedFile(generatedRoot, `articles/${article.slug}/body.html`, body);
  await writeGeneratedFile(
    generatedRoot,
    `public-root/ogp/articles/${article.slug}.png`,
    imageBytes,
  );
}

async function writeArticleIndex(generatedRoot, articles) {
  await writeGeneratedFile(
    generatedRoot,
    "article-index.json",
    `${JSON.stringify(articles.map(({ body: _body, ...article }) => article))}\n`,
  );
}

async function reservePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const { port } = server.address();
  await new Promise((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
  return port;
}

async function waitFor(predicate, message, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.fail(message);
}

async function stopServer(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  const exited = new Promise((resolve) => child.once("exit", resolve));
  child.kill("SIGTERM");
  await Promise.race([exited, new Promise((resolve) => setTimeout(resolve, 3000))]);
  if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
}

test(
  "Astro dev server applies generated route additions and deletions after a missing generated root is created",
  { timeout: 90000 },
  async (context) => {
    const fixture = await createFixture();
    context.after(() => rm(fixture.root, { recursive: true, force: true }));
    const port = await reservePort();
    const logs = [];
    const child = spawn(
      process.execPath,
      [astroCli, "dev", "--host", "127.0.0.1", "--port", String(port), "--strictPort"],
      {
        cwd: fixture.frontend,
        env: {
          ...process.env,
          BASE_URL: "https://example.test",
          ASTRO_TELEMETRY_DISABLED: "1",
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    const captureLog = (chunk) => {
      logs.push(chunk.toString());
      if (logs.join("").length > 12000) logs.splice(0, logs.length - 1);
    };
    child.stdout.on("data", captureLog);
    child.stderr.on("data", captureLog);
    context.after(async () => stopServer(child));

    const baseUrl = `http://127.0.0.1:${port}`;
    try {
      await waitFor(async () => {
        try {
          await fetch(`${baseUrl}/_generated_reload_ready`);
          return true;
        } catch {
          return false;
        }
      }, "Astro開発サーバーが起動しない");

      const alpha = createArticle("alpha", "Alpha title", "<p>ALPHA_BODY_V1</p>");
      await writeArticle(fixture.generated, alpha, Buffer.from([1, 2, 3]));
      await writeGeneratedFile(
        fixture.generated,
        "site.json",
        JSON.stringify({ title: "Example Blog", description: "Example", language: "en" }),
      );
      await writeArticleIndex(fixture.generated, [alpha]);

      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/articles/alpha/`);
          return response.status === 200 && (await response.text()).includes("ALPHA_BODY_V1");
        } catch {
          return false;
        }
      }, `新規記事 alpha のルートを取得できない\n${logs.join("")}`);

      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/ogp/articles/alpha.png`);
          return response.status === 200 &&
            Buffer.from(await response.arrayBuffer()).equals(Buffer.from([1, 2, 3]));
        } catch {
          return false;
        }
      }, `新規記事 alpha のOGP画像を取得できない\n${logs.join("")}`);

      const updatedAlpha = createArticle(
        "alpha",
        "Alpha title updated",
        "<p>ALPHA_BODY_V2</p>",
      );
      await writeArticle(fixture.generated, updatedAlpha, Buffer.from([4, 5, 6]));
      await writeGeneratedFile(
        fixture.generated,
        "site.json",
        JSON.stringify({ title: "Updated Example Blog", description: "Example", language: "en" }),
      );
      await writeArticleIndex(fixture.generated, [updatedAlpha]);

      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/articles/alpha/`);
          const html = await response.text();
          return response.status === 200 && html.includes("ALPHA_BODY_V2") &&
            html.includes("Alpha title updated") && html.includes("Updated Example Blog");
        } catch {
          return false;
        }
      }, `本文・タイトル・サイト情報の変更が反映されない\n${logs.join("")}`);

      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/ogp/articles/alpha.png`);
          return response.status === 200 &&
            Buffer.from(await response.arrayBuffer()).equals(Buffer.from([4, 5, 6]));
        } catch {
          return false;
        }
      }, `更新されたOGP画像を取得できない\n${logs.join("")}`);

      const beta = createArticle("beta", "Beta title", "<p>BETA_BODY</p>");
      await writeArticle(fixture.generated, beta, Buffer.from([7, 8, 9]));
      await writeArticleIndex(fixture.generated, [updatedAlpha, beta]);

      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/articles/beta/`);
          return response.status === 200 && (await response.text()).includes("BETA_BODY");
        } catch {
          return false;
        }
      }, `追加記事 beta のルートを再構築できない\n${logs.join("")}`);

      await writeArticleIndex(fixture.generated, [beta]);
      await waitFor(async () => {
        try {
          const response = await fetch(`${baseUrl}/articles/alpha/`);
          return response.status === 404;
        } catch {
          return false;
        }
      }, `削除記事 alpha の古いルートが無効化されない\n${logs.join("")}`);

      const indexResponse = await fetch(baseUrl);
      assert.equal(indexResponse.status, 200);
      const indexHtml = await indexResponse.text();
      assert.match(indexHtml, /Beta title/);
      assert.doesNotMatch(indexHtml, /Alpha title/);
    } finally {
      await stopServer(child);
    }
  },
);
