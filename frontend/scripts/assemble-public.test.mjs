import { strict as assert } from "node:assert";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";

import { assemblePublic } from "./assemble-public.mjs";

/**
 * なぜ実装するのか:
 * - 公開資産統合のテストごとに、実リポジトリから隔離された入力と出力を用意するため。
 *
 * 期待する動作:
 * - frontend/public、generated/public-root、および生成ルート外の所有記録を作成する。
 * - generated/public-rootにはRust生成物を配置できる初期状態を用意する。
 *
 * 使い方の概要:
 * - 各テストで返却されたパスをassemblePublicに渡す。
 */
async function createFixture() {
  const root = await mkdtemp(join(tmpdir(), "assemble-public-"));
  const paths = {
    frontendPublic: join(root, "frontend", "public"),
    publicRoot: join(root, "generated", "public-root"),
    manifest: join(root, "generated", ".frontend-public-manifest.json"),
  };

  await mkdir(paths.frontendPublic, { recursive: true });
  await mkdir(paths.publicRoot, { recursive: true });
  return paths;
}

/**
 * なぜ実装するのか:
 * - テスト入力の相対パスを作成し、資産の配置方法を各テストから分離するため。
 *
 * 期待する動作:
 * - 親ディレクトリを必要に応じて作成し、指定された内容をファイルへ保存する。
 *
 * 使い方の概要:
 * - fixtureの入力ルートと公開相対パスを渡して呼び出す。
 */
async function writeAsset(root, relativePath, content) {
  const filePath = join(root, relativePath);
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, content);
}

test("frontend/publicのファイルをgenerated/public-rootへコピーする", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "favicon.svg", "frontend asset");

  await assemblePublic(fixture);

  assert.equal(
    await readFile(join(fixture.publicRoot, "favicon.svg"), "utf8"),
    "frontend asset",
  );
});

test("既存のgenerated/public-rootにあるRust生成物を保持する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "favicon.svg", "frontend asset");
  await writeAsset(
    fixture.publicRoot,
    "ogp/articles/example.png",
    Buffer.from([0, 1, 2, 3]),
  );

  await assemblePublic(fixture);

  assert.deepEqual(
    await readFile(join(fixture.publicRoot, "ogp", "articles", "example.png")),
    Buffer.from([0, 1, 2, 3]),
  );
  assert.equal(
    await readFile(join(fixture.publicRoot, "favicon.svg"), "utf8"),
    "frontend asset",
  );
});

test("ネストしたfrontend公開ファイルのパスを維持する", async () => {
  const fixture = await createFixture();
  await writeAsset(
    fixture.frontendPublic,
    "assets/nested/deep/example.txt",
    "nested asset",
  );

  await assemblePublic(fixture);

  assert.equal(
    await readFile(
      join(fixture.publicRoot, "assets", "nested", "deep", "example.txt"),
      "utf8",
    ),
    "nested asset",
  );
});

test("frontend/publicから削除した前回コピー分だけを公開ルートから削除する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "old.txt", "old frontend asset");
  await writeAsset(fixture.publicRoot, "ogp/example.png", "rust asset");
  await assemblePublic(fixture);

  await rm(join(fixture.frontendPublic, "old.txt"));
  await assemblePublic(fixture);

  await assert.rejects(readFile(join(fixture.publicRoot, "old.txt")));
  assert.equal(
    await readFile(join(fixture.publicRoot, "ogp", "example.png"), "utf8"),
    "rust asset",
  );
});

test("同一相対パスに異なるRust生成ファイルがある場合は拒否する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "same.txt", "frontend");
  await writeAsset(fixture.publicRoot, "same.txt", "generated");

  await assert.rejects(assemblePublic(fixture), (error) => {
    assert.match(error.message, /public asset collision.*same\.txt/s);
    assert.match(error.message, new RegExp(fixture.frontendPublic));
    assert.match(error.message, new RegExp(fixture.publicRoot));
    return true;
  });
});

test("ファイルとディレクトリの衝突を拒否する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "assets", "frontend file");
  await writeAsset(fixture.publicRoot, "assets/example.txt", "generated file");

  await assert.rejects(assemblePublic(fixture), (error) => {
    assert.match(error.message, /public asset collision.*assets/s);
    assert.match(error.message, new RegExp(fixture.frontendPublic));
    assert.match(error.message, new RegExp(fixture.publicRoot));
    return true;
  });
});

test("frontend/publicが存在しない場合は明確に失敗する", async () => {
  const fixture = await createFixture();
  await rm(fixture.frontendPublic, { recursive: true });

  await assert.rejects(
    assemblePublic(fixture),
    /frontend\/public.*required|frontend\/public.*存在/s,
  );
});

test("衝突エラーの公開パスにOS依存区切りを含めない", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "nested/deep/file.txt", "frontend");
  await writeAsset(fixture.publicRoot, "nested/deep/file.txt", "generated");

  await assert.rejects(assemblePublic(fixture), (error) => {
    assert.match(error.message, /nested\/deep\/file\.txt/);
    assert.doesNotMatch(error.message, /nested\\deep\\file\.txt/);
    return true;
  });
});
