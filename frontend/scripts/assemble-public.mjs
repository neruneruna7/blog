import { createHash, randomUUID } from "node:crypto";
import {
  copyFile,
  lstat,
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const frontendRoot = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

const defaultPaths = {
  frontendPublic: join(frontendRoot, "public"),
  publicRoot: join(repositoryRoot, "generated", "public-root"),
  manifest: join(repositoryRoot, "generated", ".frontend-public-manifest.json"),
};

/**
 * なぜ実装するのか:
 * - frontend/public の静的ファイルをRust生成物と同じAstro公開ルートへ配置するため。
 *
 * 期待する動作:
 * - 既存の generated/public-root を消さず、frontend/public のファイルを統合する。
 * - frontend/public の前回コピー分だけ更新・削除し、Rust生成物との異なる内容の衝突は拒否する。
 * - 公開パスの区切りは常に `/` とし、実ファイルパスはエラーに含める。
 *
 * 使い方の概要:
 * - 通常は引数なしで `node scripts/assemble-public.mjs` を実行する。
 * - テストでは各パスを一時ディレクトリへ差し替える。
 */
export async function assemblePublic({
  frontendPublic = defaultPaths.frontendPublic,
  publicRoot = defaultPaths.publicRoot,
  manifest = defaultPaths.manifest,
} = {}) {
  const [frontendEntries, publicRootEntries, previousManifest] = await Promise.all([
    collectPublicEntries(frontendPublic, "frontend/public"),
    collectPublicEntries(publicRoot, "generated/public-root", { optional: true }),
    readManifest(manifest),
  ]);

  const frontendPaths = new Set(frontendEntries.map((entry) => entry.publicPath));
  await validateMerge(frontendEntries, publicRootEntries, previousManifest);

  await mkdir(publicRoot, { recursive: true });
  for (const entry of frontendEntries) {
    const destination = join(publicRoot, ...entry.segments);
    if (entry.kind === "directory") {
      await mkdir(destination, { recursive: true });
      continue;
    }

    await mkdir(dirname(destination), { recursive: true });
    await copyFile(entry.sourcePath, destination);
  }

  await removeStaleFrontendEntries(
    frontendPaths,
    publicRoot,
    publicRootEntries,
    previousManifest,
  );
  await writeManifest(
    manifest,
    frontendEntries.filter((entry) => entry.kind === "file"),
  );
}

/**
 * なぜ実装するのか:
 * - 入力ディレクトリの存在契約と、公開パスごとの型を統一した一覧へ変換するため。
 *
 * 期待する動作:
 * - ディレクトリを再帰走査し、ファイルとディレクトリの実パスを保持する。
 * - optional な入力だけは欠落を空一覧として返し、それ以外の欠落や型不一致を失敗させる。
 *
 * 使い方の概要:
 * - assemblePublic から frontend/public と generated/public-root の入力ごとに呼び出す。
 */
async function collectPublicEntries(root, label, { optional = false } = {}) {
  let rootStats;
  try {
    rootStats = await lstat(root);
  } catch (error) {
    if (optional && error.code === "ENOENT") {
      return [];
    }
    if (error.code === "ENOENT") {
      throw new Error(`${label} is required but does not exist: ${root}`);
    }
    throw new Error(`failed to inspect ${label}: ${root}: ${error.message}`, {
      cause: error,
    });
  }

  if (!rootStats.isDirectory()) {
    throw new Error(`${label} must be a directory: ${root}`);
  }

  return walkPublicDirectory(root, []);
}

/**
 * なぜ実装するのか:
 * - OSのパス表現と、ブラウザへ公開するURLパスを分離するため。
 *
 * 期待する動作:
 * - 各エントリの相対パスをセグメント配列と `/` 区切りの公開パスで保持する。
 * - 未対応のファイル型を暗黙にコピーせず、入力として拒否する。
 *
 * 使い方の概要:
 * - collectPublicEntries から入力ルートと再帰中のセグメントを渡して呼び出す。
 */
async function walkPublicDirectory(currentDirectory, parentSegments) {
  const entries = [];
  const directoryEntries = await readdir(currentDirectory, { withFileTypes: true });

  for (const directoryEntry of directoryEntries) {
    const segments = [...parentSegments, directoryEntry.name];
    const sourcePath = join(currentDirectory, directoryEntry.name);

    if (directoryEntry.isDirectory()) {
      entries.push({
        kind: "directory",
        publicPath: segments.join("/"),
        segments,
        sourcePath,
      });
      entries.push(...(await walkPublicDirectory(sourcePath, segments)));
      continue;
    }

    if (directoryEntry.isFile()) {
      entries.push({
        kind: "file",
        publicPath: segments.join("/"),
        segments,
        sourcePath,
      });
      continue;
    }

    throw new Error(`unsupported public asset type: ${sourcePath}`);
  }

  return entries;
}

/**
 * なぜ実装するのか:
 * - Astro公開ルートをRust成果物ごと再構築せず、所有者が判別できる範囲だけ安全に統合するため。
 *
 * 期待する動作:
 * - ディレクトリ同士と内容が同じファイルは統合する。
 * - 前回frontend/publicからコピーしたファイルは更新を許可する。
 * - Rust生成物など、frontend/publicが所有していない異なるファイルとの衝突は拒否する。
 *
 * 使い方の概要:
 * - assemblePublic が両ルートの一覧と前回の所有記録を渡して呼び出す。
 */
async function validateMerge(frontendEntries, publicRootEntries, previousManifest) {
  const publicRootByPath = new Map(
    publicRootEntries.map((entry) => [entry.publicPath, entry]),
  );

  for (const frontendEntry of frontendEntries) {
    const existing = publicRootByPath.get(frontendEntry.publicPath);
    if (!existing) {
      continue;
    }

    if (existing.kind !== frontendEntry.kind) {
      throw createCollisionError(frontendEntry, existing);
    }
    if (frontendEntry.kind === "directory") {
      continue;
    }

    const [frontendBytes, existingBytes] = await Promise.all([
      readFile(frontendEntry.sourcePath),
      readFile(existing.sourcePath),
    ]);
    if (frontendBytes.equals(existingBytes)) {
      continue;
    }

    const previousDigest = previousManifest.get(frontendEntry.publicPath);
    if (previousDigest && sha256(existingBytes) === previousDigest) {
      continue;
    }

    throw createCollisionError(frontendEntry, existing);
  }
}

/**
 * なぜ実装するのか:
 * - frontend/public から削除されたファイルを除去しつつ、Rust側が更新したファイルを誤削除しないため。
 *
 * 期待する動作:
 * - 前回コピーした内容と現在の公開ルートの内容が一致するファイルだけを削除する。
 * - Rust側などが置き換えたファイルやfrontend/publicの現行ファイルは保持する。
 *
 * 使い方の概要:
 * - assemblePublic がコピー後に前回の所有記録と現在の公開ルートを渡して呼び出す。
 */
async function removeStaleFrontendEntries(
  frontendPaths,
  publicRoot,
  publicRootEntries,
  previousManifest,
) {
  const publicRootByPath = new Map(
    publicRootEntries.map((entry) => [entry.publicPath, entry]),
  );

  for (const [publicPath, previousDigest] of previousManifest) {
    if (frontendPaths.has(publicPath)) {
      continue;
    }

    const existing = publicRootByPath.get(publicPath);
    if (!existing || existing.kind !== "file") {
      continue;
    }

    const existingBytes = await readFile(existing.sourcePath);
    if (sha256(existingBytes) === previousDigest) {
      await rm(join(publicRoot, ...publicPath.split("/")));
    }
  }
}

/**
 * なぜ実装するのか:
 * - frontend/public が前回配置したファイルを、Rust生成物と区別して扱うため。
 *
 * 期待する動作:
 * - 記録がなければ空の所有記録を返し、壊れたJSONや不正なパスを明示的に拒否する。
 *
 * 使い方の概要:
 * - assemblePublic が統合前に一度だけ呼び出す。
 */
async function readManifest(path) {
  let contents;
  try {
    contents = await readFile(path, "utf8");
  } catch (error) {
    if (error.code === "ENOENT") {
      return new Map();
    }
    throw new Error(`failed to read frontend public manifest: ${path}: ${error.message}`, {
      cause: error,
    });
  }

  let parsed;
  try {
    parsed = JSON.parse(contents);
  } catch (error) {
    throw new Error(`invalid frontend public manifest: ${path}: ${error.message}`, {
      cause: error,
    });
  }

  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`invalid frontend public manifest: expected an object: ${path}`);
  }

  const entries = new Map();
  for (const [publicPath, digest] of Object.entries(parsed)) {
    const segments = publicPath.split("/");
    if (
      segments.some(
        (segment) =>
          segment === "" ||
          segment === "." ||
          segment === ".." ||
          segment.includes("\\") ||
          segment.includes(":"),
      ) ||
      typeof digest !== "string" ||
      !/^[a-f0-9]{64}$/.test(digest)
    ) {
      throw new Error(`invalid frontend public manifest entry: ${publicPath}`);
    }
    entries.set(publicPath, digest);
  }

  return entries;
}

/**
 * なぜ実装するのか:
 * - コピーした静的ファイルの所有情報を、公開URLに含めず再実行間で保持するため。
 *
 * 期待する動作:
 * - frontend/publicのファイルごとのSHA-256を安定したJSON形式で記録する。
 * - 一時ファイルを完成させてから置き換え、途中書き込みのJSONを残さない。
 *
 * 使い方の概要:
 * - assemblePublic が統合と stale ファイル削除の後に呼び出す。
 */
async function writeManifest(path, entries) {
  const manifestEntries = await Promise.all(
    entries.map(async (entry) => [entry.publicPath, sha256(await readFile(entry.sourcePath))]),
  );
  manifestEntries.sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));

  const manifest = Object.fromEntries(manifestEntries);
  const temporaryPath = `${path}.${randomUUID()}.tmp`;
  await mkdir(dirname(path), { recursive: true });
  await writeFile(temporaryPath, `${JSON.stringify(manifest, null, 2)}\n`);
  try {
    await rename(temporaryPath, path);
  } catch (error) {
    await rm(temporaryPath, { force: true });
    throw error;
  }
}

function sha256(contents) {
  return createHash("sha256").update(contents).digest("hex");
}

function createCollisionError(frontendEntry, publicRootEntry) {
  return new Error(
    `public asset collision: public path="${frontendEntry.publicPath}"; ` +
      `${frontendEntry.sourcePath} (${frontendEntry.kind}) conflicts with ` +
      `${publicRootEntry.sourcePath} (${publicRootEntry.kind})`,
  );
}

const entryPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (entryPath === fileURLToPath(import.meta.url)) {
  assemblePublic().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
