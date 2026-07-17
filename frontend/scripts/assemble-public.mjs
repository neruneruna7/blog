import { copyFile, lstat, mkdir, readdir, rm } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const frontendRoot = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

const defaultPaths = {
  frontendPublic: join(frontendRoot, "public"),
  generatedPublic: join(repositoryRoot, "generated", "public"),
  outputRoot: join(repositoryRoot, "generated", "public-root"),
};

/**
 * なぜ実装するのか:
 * - Rust生成資産とfrontend管理資産を、Astroが読む単一の公開ルートへ統合するため。
 *
 * 期待する動作:
 * - frontend/public の欠落は空として扱い、generated/public の欠落は失敗させる。
 * - 衝突を検証してから出力ルートを空にし、成功時だけ両入力を完全にコピーする。
 * - 公開パスの区切りは常に `/` とし、実ファイルパスはエラーに含める。
 *
 * 使い方の概要:
 * - 通常は引数なしで `node scripts/assemble-public.mjs` を実行する。
 * - テストでは `frontendPublic`、`generatedPublic`、`outputRoot` を一時ディレクトリへ差し替える。
 */
export async function assemblePublic({
  frontendPublic = defaultPaths.frontendPublic,
  generatedPublic = defaultPaths.generatedPublic,
  outputRoot = defaultPaths.outputRoot,
} = {}) {
  const [frontendEntries, generatedEntries] = await Promise.all([
    collectPublicEntries(frontendPublic, "frontend/public", { optional: true }),
    collectPublicEntries(generatedPublic, "generated/public"),
  ]);

  const entries = mergePublicEntries(frontendEntries, generatedEntries);

  await rm(outputRoot, { recursive: true, force: true });
  await mkdir(outputRoot, { recursive: true });

  for (const entry of entries) {
    const destination = join(outputRoot, ...entry.segments);
    if (entry.kind === "directory") {
      await mkdir(destination, { recursive: true });
      continue;
    }

    await mkdir(dirname(destination), { recursive: true });
    await copyFile(entry.sourcePath, destination);
  }
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
 * - assemblePublic から、frontend/public または generated/public の入力ごとに呼び出す。
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
 * - 二つの所有境界を統合する前に、上書きによる公開資産の消失を検出するため。
 *
 * 期待する動作:
 * - 同じ相対パスのディレクトリ同士だけを統合し、ファイル同士または型違いを拒否する。
 * - 衝突時に公開パスと双方の実パスを含むエラーを返す。
 *
 * 使い方の概要:
 * - assemblePublic が各入力の走査結果を渡し、コピー前に一度だけ呼び出す。
 */
function mergePublicEntries(...entryGroups) {
  const merged = new Map();

  for (const entries of entryGroups) {
    for (const entry of entries) {
      const previous = merged.get(entry.publicPath);
      if (previous && (previous.kind !== "directory" || entry.kind !== "directory")) {
        throw new Error(
          `public asset collision: public path="${entry.publicPath}"; ` +
            `${previous.sourcePath} (${previous.kind}) conflicts with ` +
            `${entry.sourcePath} (${entry.kind})`,
        );
      }
      if (!previous) {
        merged.set(entry.publicPath, entry);
      }
    }
  }

  return [...merged.values()];
}

const entryPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (entryPath === fileURLToPath(import.meta.url)) {
  assemblePublic().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
