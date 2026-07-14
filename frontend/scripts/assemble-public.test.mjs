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
 * - frontend/public と generated/public を一時ディレクトリ配下に作成する。
 * - generated/public は、入力欠落契約を検証できるよう初期状態では存在させる。
 *
 * 使い方の概要:
 * - 各テストで返却されたパスを assemblePublic に渡す。
 */
async function createFixture() {
  const root = await mkdtemp(join(tmpdir(), "assemble-public-"));
  const paths = {
    frontendPublic: join(root, "frontend", "public"),
    generatedPublic: join(root, "generated", "public"),
    outputRoot: join(root, "generated", "public-root"),
  };

  await mkdir(paths.frontendPublic, { recursive: true });
  await mkdir(paths.generatedPublic, { recursive: true });
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
 * - fixture の入力ルートと公開相対パスを渡して呼び出す。
 */
async function writeAsset(root, relativePath, content) {
  const filePath = join(root, relativePath);
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, content);
}

test("frontend/public のファイルを出力へコピーする", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "favicon.svg", "frontend asset");

  await assemblePublic(fixture);

  assert.equal(
    await readFile(join(fixture.outputRoot, "favicon.svg"), "utf8"),
    "frontend asset",
  );
});

test("generated/public のファイルを出力へコピーする", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.generatedPublic, "robots.txt", "generated asset");

  await assemblePublic(fixture);

  assert.equal(
    await readFile(join(fixture.outputRoot, "robots.txt"), "utf8"),
    "generated asset",
  );
});

test("ネストしたディレクトリを維持する", async () => {
  const fixture = await createFixture();
  await writeAsset(
    fixture.generatedPublic,
    "assets/nested/deep/example.txt",
    "nested asset",
  );

  await assemblePublic(fixture);

  assert.equal(
    await readFile(
      join(fixture.outputRoot, "assets", "nested", "deep", "example.txt"),
      "utf8",
    ),
    "nested asset",
  );
});

test("同一相対パスのファイル衝突を拒否する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "same.txt", "frontend");
  await writeAsset(fixture.generatedPublic, "same.txt", "generated");

  await assert.rejects(
    assemblePublic(fixture),
    (error) => {
      assert.match(error.message, /public asset collision.*same\.txt/s);
      assert.match(error.message, new RegExp(fixture.frontendPublic));
      assert.match(error.message, new RegExp(fixture.generatedPublic));
      return true;
    },
  );
});

test("ファイルとディレクトリの衝突を拒否する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "assets", "frontend file");
  await writeAsset(fixture.generatedPublic, "assets/example.txt", "generated file");

  await assert.rejects(
    assemblePublic(fixture),
    (error) => {
      assert.match(error.message, /public asset collision.*assets/s);
      assert.match(error.message, new RegExp(fixture.frontendPublic));
      assert.match(error.message, new RegExp(fixture.generatedPublic));
      return true;
    },
  );
});

test("再実行時に前回出力だけに存在した古いファイルを削除する", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "old.txt", "old");
  await assemblePublic(fixture);

  await rm(join(fixture.frontendPublic, "old.txt"));
  await writeAsset(fixture.frontendPublic, "new.txt", "new");
  await assemblePublic(fixture);

  await assert.rejects(readFile(join(fixture.outputRoot, "old.txt")));
  assert.equal(await readFile(join(fixture.outputRoot, "new.txt"), "utf8"), "new");
});

test("OGP画像を公開ルートの ogp/articles へコピーする", async () => {
  const fixture = await createFixture();
  await writeAsset(
    fixture.generatedPublic,
    "ogp/articles/example.png",
    Buffer.from([0, 1, 2, 3]),
  );

  await assemblePublic(fixture);

  assert.deepEqual(
    await readFile(join(fixture.outputRoot, "ogp", "articles", "example.png")),
    Buffer.from([0, 1, 2, 3]),
  );
});

test("frontend/public が存在しない場合は空として扱う", async () => {
  const fixture = await createFixture();
  await rm(fixture.frontendPublic, { recursive: true });
  await writeAsset(fixture.generatedPublic, "generated.txt", "generated");

  await assemblePublic(fixture);

  assert.equal(
    await readFile(join(fixture.outputRoot, "generated.txt"), "utf8"),
    "generated",
  );
});

test("generated/public が存在しない場合は明確に失敗する", async () => {
  const fixture = await createFixture();
  await rm(fixture.generatedPublic, { recursive: true });

  await assert.rejects(
    assemblePublic(fixture),
    /generated\/public.*required|generated\/public.*存在/s,
  );
});

test("衝突エラーの公開パスにOS依存区切りを含めない", async () => {
  const fixture = await createFixture();
  await writeAsset(fixture.frontendPublic, "nested/deep/file.txt", "frontend");
  await writeAsset(fixture.generatedPublic, "nested/deep/file.txt", "generated");

  await assert.rejects(
    assemblePublic(fixture),
    (error) => {
      assert.match(error.message, /nested\/deep\/file\.txt/);
      assert.doesNotMatch(error.message, /nested\\deep\\file\.txt/);
      return true;
    },
  );
});
