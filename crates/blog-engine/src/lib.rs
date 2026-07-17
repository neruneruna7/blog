//! なぜ実装するのか:
//! - Typstブログ向けメタデータ生成の公開境界をまとめるため。
//!
//! 期待する動作:
//! - TOML入力を検証し、OGP PNG、生成JSON、記事本文HTML fragmentへ変換する機能を提供する。
//! - 全処理が成功した場合だけステージング成果物を公開出力へ確定する。
//!
//! 使い方の概要:
//! - CLIやテストから `build` または `load_normalized_articles` を呼び出す。

pub mod article;
pub mod config;
pub mod error;
pub mod html_postprocess;
pub mod new_article;
pub mod ogp;
pub mod ogp_render;
pub mod output;
pub mod typst_compile;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use article::{normalize_article_sources, normalize_articles, Article};
use config::{
    load_article_config_files, load_article_configs, load_site_config,
    load_site_config_with_base_url,
};
use error::BlogEngineError;
use ogp::{
    build_ogp_public_path, generate_ogp_content_hash, OgpImageMeta, OgpRenderInput,
    OgpRenderSettings,
};
use ogp_render::{OgpRenderer, TypstOgpRenderer, OGP_HEIGHT_PX, OGP_WIDTH_PX};
use output::{write_article_outputs, write_site_output, SiteOutput};
use typst_compile::{write_article_body_html, CliTypstRunner, TypstRunner};

/// なぜ実装するのか:
/// - OGP生成の観測可能なコストを、ビルド完了後のログとテストで共有するため。
///
/// 期待する動作:
/// - 記事ごとの出力数、総描画時間、総バイト数、最大ファイルサイズを保持する。
/// - 数値はステージング出力へ書かれたPNGの実測値から集計する。
///
/// 使い方の概要:
/// - BuildReport.ogp から参照し、ビルド成功時にログへ出力する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OgpBuildStats {
    pub images_generated: usize,
    pub render_time: Duration,
    pub total_size: u64,
    pub largest_file: u64,
}

/// なぜ実装するのか:
/// - テストとビルド呼び出し側が、生成物に付随する計測結果を確認できるようにするため。
///
/// 期待する動作:
/// - OGP成果物がすべて確定した成功ビルドだけがレポートを返す。
/// - 本文やJSONの失敗時にはレポートを返さず、エラーを返す。
///
/// 使い方の概要:
/// - build_with_runners の戻り値からOGP統計を取得する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildReport {
    pub ogp: OgpBuildStats,
}

/// ビルド時にファイル設定へ適用する上書き値である。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildOptions {
    pub base_url: Option<String>,
}

/// なぜ実装するのか:
/// - CLIとテストの両方から同じビルド境界を呼び出せるようにするため。
///
/// 期待する動作:
/// - 入力ルートからサイト設定、記事設定、記事本文を読み、OGPを含む生成物を出力ルートへ書き込む。
/// - OGP rendererには本番のTypstOgpRendererを使う。
///
/// 使い方の概要:
/// - `build("typst-src", "generated")` のように呼び出す。
pub fn build(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
) -> Result<(), BlogEngineError> {
    build_with_options(source_root, output_root, BuildOptions::default())
}

/// CLIなどの呼び出し元が指定した設定を適用してコンテンツを生成する。
pub fn build_with_options(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    options: BuildOptions,
) -> Result<(), BlogEngineError> {
    let source_root = source_root.as_ref();
    let runner = CliTypstRunner::new(source_root);
    let ogp_renderer =
        TypstOgpRenderer::new(source_root.join("commons/ogp/template.typ"), source_root);
    let postprocess_context = html_postprocess::PostprocessContext::production();

    build_with_runners_and_postprocess_context(
        source_root,
        output_root,
        &runner,
        &ogp_renderer,
        &postprocess_context,
        options.base_url.as_deref(),
    )
    .map(|_| ())
}
/// なぜ実装するのか:
/// - 本番ではTypst CLIを使い、テストでは外部バイナリに依存しないrunnerを注入するため。
///
/// 期待する動作:
/// - 入力ルートからサイト設定、記事設定、記事本文を読み、OGPを含むJSONとbody.htmlを出力する。
/// - 本番ビルドでは使わず、本文runnerだけを差し替える既存テスト境界を維持する。
///
/// 使い方の概要:
/// - 通常は `build` を使い、テストや特殊な実行境界だけrunnerを明示する。
pub fn build_with_typst_runner(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    runner: &impl TypstRunner,
) -> Result<(), BlogEngineError> {
    let ogp_renderer = DeterministicOgpRenderer;
    let postprocess_context = html_postprocess::PostprocessContext::production();
    build_with_runners_and_postprocess_context(
        source_root,
        output_root,
        runner,
        &ogp_renderer,
        &postprocess_context,
        None,
    )
    .map(|_| ())
}

/// なぜ実装するのか:
/// - 本番のTypst runnerとOGP rendererを、テスト用fakeへ同時に差し替えるため。
///
/// 期待する動作:
/// - 検証済み記事を公開順に処理し、OGP、本文、最終JSONの順で成果物を確定する。
/// - 全処理が成功した場合だけ BuildReport を返し、失敗時は既存出力を変更しない。
///
/// 使い方の概要:
/// - build_with_runners(source, output, &fake_typst, &fake_ogp) のように呼び出す。
pub fn build_with_runners(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    typst_runner: &impl TypstRunner,
    ogp_renderer: &impl OgpRenderer,
) -> Result<BuildReport, BlogEngineError> {
    let postprocess_context = html_postprocess::PostprocessContext::production();
    build_with_runners_and_postprocess_context(
        source_root,
        output_root,
        typst_runner,
        ogp_renderer,
        &postprocess_context,
        None,
    )
}

/// なぜ実装するのか:
/// - Typst実行とHTML後処理の外部境界を個別にテスト可能にするため。
///
/// 期待する動作:
/// - OGP生成、Typst変換、本文fragment後処理、JSON書き出しをステージング内で実行する。
/// - 指定された後処理文脈を本文へ適用し、全処理成功後に出力を確定する。
///
/// 使い方の概要:
/// - 通常は `build_with_typst_runner` を使い、後処理を差し替えるテストだけで使う。
pub fn build_with_typst_runner_and_postprocess_context(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    runner: &impl TypstRunner,
    postprocess_context: &html_postprocess::PostprocessContext,
) -> Result<(), BlogEngineError> {
    let ogp_renderer = DeterministicOgpRenderer;
    build_with_runners_and_postprocess_context(
        source_root,
        output_root,
        runner,
        &ogp_renderer,
        postprocess_context,
        None,
    )
    .map(|_| ())
}

/// なぜ実装するのか:
/// - runnerを注入する公開関数と、本文後処理を差し替える既存境界を同じ実装へ集約するため。
///
/// 期待する動作:
/// - ステージングで全生成を完了し、成功時だけ公開出力を置換する。
/// - 成功時はOGP統計を含むビルドレポートを返す。
///
/// 使い方の概要:
/// - 公開ビルド関数からTypst runner、OGP renderer、後処理文脈を渡して呼び出す。
fn build_with_runners_and_postprocess_context(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    typst_runner: &impl TypstRunner,
    ogp_renderer: &impl OgpRenderer,
    postprocess_context: &html_postprocess::PostprocessContext,
    base_url: Option<&str>,
) -> Result<BuildReport, BlogEngineError> {
    let source_root = source_root.as_ref();
    let output_root = output_root.as_ref();
    let staging_root = prepare_staging_output(output_root)?;

    let result = build_into_staging(
        source_root,
        &staging_root,
        typst_runner,
        ogp_renderer,
        postprocess_context,
        base_url,
    );
    let report = match result {
        Ok(report) => report,
        Err(error) => {
            let _ = remove_path(&staging_root);
            return Err(error);
        }
    };

    if let Err(error) = commit_staging_output(&staging_root, output_root) {
        let _ = remove_path(&staging_root);
        return Err(error);
    }

    print_ogp_stats(report.ogp);
    Ok(report)
}

/// なぜ実装するのか:
/// - 公開出力へ確定する前の作業領域で、ビルド順序を一箇所に固定するため。
///
/// 期待する動作:
/// - サイト設定、記事モデル、OGP、本文、最終JSONの順に処理する。
/// - 途中失敗時はステージング外へJSONを漏らさない。
///
/// 使い方の概要:
/// - build_with_runners_and_postprocess_context から呼び出す。
fn build_into_staging(
    source_root: &Path,
    staging_root: &Path,
    typst_runner: &impl TypstRunner,
    ogp_renderer: &impl OgpRenderer,
    postprocess_context: &html_postprocess::PostprocessContext,
    base_url: Option<&str>,
) -> Result<BuildReport, BlogEngineError> {
    let site = load_site_config_with_base_url(source_root, base_url)?;
    let article_sources = load_normalized_article_sources(source_root)?;
    let articles = article_sources
        .iter()
        .map(|source| source.article.clone())
        .collect::<Vec<Article>>();

    let (ogp_images, ogp_stats) =
        render_ogp_images(staging_root, &site, &article_sources, ogp_renderer)?;
    for article in &article_sources {
        write_article_body_html(staging_root, article, typst_runner, postprocess_context)?;
    }

    write_site_output(staging_root, &site)?;
    write_article_outputs(staging_root, &articles, &ogp_images)?;

    Ok(BuildReport { ogp: ogp_stats })
}

/// なぜ実装するのか:
/// - OGP成果物と記事JSONが参照する公開パスを同じ処理で確定するため。
///
/// 期待する動作:
/// - 記事を公開順に逐次処理し、同一ハッシュは一度だけレンダリングする。
/// - PNGの実測サイズを集計し、JSONへ渡す画像メタデータを返す。
///
/// 使い方の概要:
/// - ステージング出力ルート、検証済みサイト・記事、注入済みrendererを渡す。
fn render_ogp_images(
    output_root: &Path,
    site: &SiteOutput,
    article_sources: &[article::ArticleSource],
    renderer: &impl OgpRenderer,
) -> Result<(Vec<OgpImageMeta>, OgpBuildStats), BlogEngineError> {
    let ogp_output_dir = output_root.join("public/ogp/articles");
    fs::create_dir_all(&ogp_output_dir)
        .map_err(|source| BlogEngineError::io(&ogp_output_dir, source))?;
    let started_at = Instant::now();
    let mut rendered_by_hash = HashMap::<String, RenderedOgp>::new();
    let mut images = Vec::with_capacity(article_sources.len());
    let mut total_size = 0_u64;
    let mut largest_file = 0_u64;

    for source in article_sources {
        let input = OgpRenderInput::from_models(site, &source.article);
        let pending_path = ogp_output_dir.join(format!("{}.pending.png", source.article.slug));
        let planned_hash = renderer.content_hash(&input, &pending_path)?;

        let rendered = if let Some(content_hash) = planned_hash {
            let final_path = ogp_output_dir.join(format!(
                "{}-{}.png",
                source.article.slug,
                content_hash.chars().take(12).collect::<String>()
            ));

            if let Some(cached) = rendered_by_hash.get(&content_hash) {
                fs::copy(&cached.file_path, &final_path)
                    .map_err(|source| BlogEngineError::io(&final_path, source))?;
                let mut image = cached.image.clone();
                image.path = build_ogp_public_path(&source.article.slug, &content_hash);
                RenderedOgp {
                    file_path: final_path,
                    image,
                    byte_size: cached.byte_size,
                }
            } else {
                let artifact = renderer.render(&input, &final_path)?;
                let rendered =
                    rendered_artifact(artifact, &final_path, &source.article.slug, &content_hash)?;
                rendered_by_hash.insert(content_hash, rendered.clone());
                rendered
            }
        } else {
            let artifact = renderer.render(&input, &pending_path)?;
            let content_hash = artifact.content_hash.clone();
            let final_path = ogp_output_dir.join(format!(
                "{}-{}.png",
                source.article.slug,
                content_hash.chars().take(12).collect::<String>()
            ));
            if artifact.file_path != pending_path {
                return Err(BlogEngineError::OgpArtifactPathMismatch {
                    expected: pending_path,
                    actual: artifact.file_path,
                });
            }
            fs::rename(&pending_path, &final_path)
                .map_err(|source| BlogEngineError::io(&final_path, source))?;
            rendered_artifact(artifact, &final_path, &source.article.slug, &content_hash)?
        };

        total_size += rendered.byte_size;
        largest_file = largest_file.max(rendered.byte_size);
        images.push(rendered.image);
    }

    Ok((
        images,
        OgpBuildStats {
            images_generated: article_sources.len(),
            render_time: started_at.elapsed(),
            total_size,
            largest_file,
        },
    ))
}

/// なぜ実装するのか:
/// - rendererの成功結果を、実在ファイルのサイズと記事公開パスへ正規化するため。
///
/// 期待する動作:
/// - 期待ハッシュがある場合は成果物ハッシュの一致を検証する。
/// - 指定出力先のPNGを確認してから記事メタデータ用の値を返す。
///
/// 使い方の概要:
/// - 事前ハッシュあり・なしのOGP生成分岐から呼び出す。
fn rendered_artifact(
    artifact: ogp::OgpArtifact,
    output_path: &Path,
    slug: &str,
    expected_hash: &str,
) -> Result<RenderedOgp, BlogEngineError> {
    if artifact.content_hash != expected_hash {
        return Err(BlogEngineError::OgpArtifactHashMismatch {
            output_path: output_path.to_path_buf(),
            expected: expected_hash.to_owned(),
            actual: artifact.content_hash,
        });
    }
    if artifact.file_path != output_path {
        return Err(BlogEngineError::OgpArtifactPathMismatch {
            expected: output_path.to_path_buf(),
            actual: artifact.file_path,
        });
    }
    let byte_size = fs::metadata(output_path)
        .map_err(|source| BlogEngineError::io(output_path, source))?
        .len();
    let mut image = artifact.image;
    image.path = build_ogp_public_path(slug, expected_hash);

    Ok(RenderedOgp {
        file_path: output_path.to_path_buf(),
        image,
        byte_size,
    })
}

/// なぜ実装するのか:
/// - 同一コンテンツハッシュを持つ記事のファイル生成を一度に制限するため。
///
/// 期待する動作:
/// - 一度Typstで生成したPNGを保持し、別slugの記事には同じバイト列を複製する。
/// - 公開パスとファイルシステム上のパスは記事ごとに再構築する。
///
/// 使い方の概要:
/// - render_ogp_images のビルド内キャッシュとしてのみ使う。
#[derive(Debug, Clone)]
struct RenderedOgp {
    file_path: PathBuf,
    image: OgpImageMeta,
    byte_size: u64,
}

/// なぜ実装するのか:
/// - 生成処理中の失敗から既存の公開成果物を保護するため。
///
/// 期待する動作:
/// - 既存出力と同じ親ディレクトリに空のnextディレクトリを作る。
/// - 前回残ったnextディレクトリだけを処理開始前に除去する。
///
/// 使い方の概要:
/// - ビルド処理の最初に出力ルートを渡す。
fn prepare_staging_output(output_root: &Path) -> Result<PathBuf, BlogEngineError> {
    let staging_root = sibling_path(output_root, ".next");
    remove_path(&staging_root)?;
    fs::create_dir_all(&staging_root)
        .map_err(|source| BlogEngineError::io(&staging_root, source))?;
    Ok(staging_root)
}

/// なぜ実装するのか:
/// - 全処理成功後にだけ公開出力を更新し、旧成果物を一括で置き換えるため。
///
/// 期待する動作:
/// - 既存出力を一時退避してからnextディレクトリを公開出力へ移動する。
/// - 移動に失敗した場合は可能な限り旧出力を復元する。
///
/// 使い方の概要:
/// - JSONと本文の書き出しが成功した後に呼び出す。
fn commit_staging_output(staging_root: &Path, output_root: &Path) -> Result<(), BlogEngineError> {
    let backup_root = sibling_path(output_root, ".previous");
    remove_path(&backup_root)?;

    if output_root.exists() {
        fs::rename(output_root, &backup_root)
            .map_err(|source| BlogEngineError::io(output_root, source))?;
    }

    if let Err(source) = fs::rename(staging_root, output_root) {
        if backup_root.exists() {
            let _ = fs::rename(&backup_root, output_root);
        }
        return Err(BlogEngineError::io(output_root, source));
    }

    remove_path(&backup_root)
}

/// なぜ実装するのか:
/// - 出力ルートとステージング・バックアップの名前を同じ親の下で決めるため。
///
/// 期待する動作:
/// - OS依存の区切り文字を追加せず、出力ルートの兄弟パスを返す。
///
/// 使い方の概要:
/// - nextまたはpreviousの接尾辞を渡して内部の置換処理で使う。
fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "generated".to_owned());
    path.with_file_name(format!("{name}{suffix}"))
}

/// なぜ実装するのか:
/// - ディレクトリと一時ファイルを同じ後始末境界で扱うため。
///
/// 期待する動作:
/// - 対象がなければ成功し、存在すればディレクトリまたはファイルを除去する。
///
/// 使い方の概要:
/// - ステージング開始前、失敗時、置換完了後に呼び出す。
fn remove_path(path: &Path) -> Result<(), BlogEngineError> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|source| BlogEngineError::io(path, source))
    } else {
        fs::remove_file(path).map_err(|source| BlogEngineError::io(path, source))
    }
}

/// なぜ実装するのか:
/// - Cloudflare PagesのビルドログからOGP生成コストを観測できるようにするため。
///
/// 期待する動作:
/// - 指定された4項目を固定ラベルで標準出力へ出力する。
///
/// 使い方の概要:
/// - 成功したビルドレポートを公開出力へ確定した直後に呼び出す。
fn print_ogp_stats(stats: OgpBuildStats) {
    println!("OGP images generated: {}", stats.images_generated);
    println!("OGP render time: {:?}", stats.render_time);
    println!("OGP total size: {} bytes", stats.total_size);
    println!("OGP largest file: {} bytes", stats.largest_file);
}

/// なぜ実装するのか:
/// - 既存の本文runner専用テスト境界を壊さず、JSON契約を必須OGPへ移行するため。
///
/// 期待する動作:
/// - 実Typstを起動せず、決定的なテスト用PNG成果物を生成する。
/// - 本番の build からは参照されず、本文runnerだけを差し替える既存テスト関数で使われる。
///
/// 使い方の概要:
/// - build_with_typst_runner 内部からのみ使う。
#[derive(Debug, Clone, Copy)]
struct DeterministicOgpRenderer;

impl OgpRenderer for DeterministicOgpRenderer {
    fn content_hash(
        &self,
        input: &OgpRenderInput,
        _output_path: &Path,
    ) -> Result<Option<String>, BlogEngineError> {
        let settings = OgpRenderSettings::new("test-ogp-v1", "test-template", 1200, 630, 72);
        Ok(Some(generate_ogp_content_hash(input, &settings)))
    }

    fn render(
        &self,
        input: &OgpRenderInput,
        output_path: &Path,
    ) -> Result<ogp::OgpArtifact, BlogEngineError> {
        let bytes = deterministic_png();
        fs::write(output_path, &bytes)
            .map_err(|source| BlogEngineError::io(output_path, source))?;
        let content_hash = self
            .content_hash(input, output_path)?
            .expect("deterministic renderer must provide a content hash");
        Ok(ogp::OgpArtifact {
            file_path: output_path.to_path_buf(),
            image: OgpImageMeta {
                path: build_ogp_public_path("test", &content_hash),
                width: OGP_WIDTH_PX,
                height: OGP_HEIGHT_PX,
                mime_type: "image/png".to_owned(),
                alt: format!("{} | {}", input.article_title, input.site_title),
            },
            content_hash,
            byte_size: bytes.len() as u64,
        })
    }
}

/// なぜ実装するのか:
/// - 本文runner専用の既存テスト境界に、JSON契約を満たすPNGバイト列を供給するため。
///
/// 期待する動作:
/// - OGP契約のPNGシグネチャと1200×630のIHDRを含む決定的なバイト列を返す。
///
/// 使い方の概要:
/// - DeterministicOgpRenderer のテスト用renderから呼び出す。
fn deterministic_png() -> Vec<u8> {
    let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
    bytes.extend_from_slice(&13_u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&OGP_WIDTH_PX.to_be_bytes());
    bytes.extend_from_slice(&OGP_HEIGHT_PX.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

/// なぜ実装するのか:
/// - 記事メタデータの正規化結果を、出力を伴わずにテストできるようにするため。
///
/// 期待する動作:
/// - ArticleConfig.toml群を読み、日時型を保持する検証済み `Article` を公開順で返す。
///
/// 使い方の概要:
/// - テストや将来の別出力処理から、記事一覧だけが必要な場合に呼び出す。
pub fn load_normalized_articles(
    source_root: impl AsRef<Path>,
) -> Result<Vec<Article>, BlogEngineError> {
    let article_configs = load_article_configs(source_root.as_ref())?;
    normalize_articles(article_configs)
}

fn load_normalized_article_sources(
    source_root: impl AsRef<Path>,
) -> Result<Vec<article::ArticleSource>, BlogEngineError> {
    let article_files = load_article_config_files(source_root.as_ref())?;
    normalize_article_sources(article_files)
}
