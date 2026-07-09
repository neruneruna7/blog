use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::article::ArticleManifest;
use crate::error::{BlogError, PathContext, Result};
use crate::manifest::{BuildInput, load_manifest};
use crate::paths::ProjectPaths;
use crate::render::{ArticlePage, article_url, escape_xml, render_article, render_index};
use crate::typst::TypstCompiler;

/// ビルド結果の主要パスである。
///
/// # なぜ実装するのか
/// テストとCLIが，生成された公開ディレクトリとキャッシュディレクトリを確認できるようにするためである。
///
/// # 期待する動作
/// build完了後の`public/`と`.blog-cache/`を指す。
///
/// # 使い方の概要
/// `build_site`の戻り値として受け取り，存在確認や後続処理に使う。
#[derive(Clone, Debug)]
pub struct BuildOutput {
    pub public_dir: PathBuf,
    pub cache_dir: PathBuf,
}

/// 静的サイトを生成する。
///
/// # なぜ実装するのか
/// Typstソースとサイト資産から，ホスティング可能な`public/`だけを生成するためである。
///
/// # 期待する動作
/// 設定検証，Typstコンパイル，Askama描画，assetコピー，feed，sitemap生成を行う。
///
/// # 使い方の概要
/// CLIの`build`または統合テストから，rootと`TypstCompiler`実装を渡して呼び出す。
pub fn build_site(root: impl AsRef<Path>, typst: &dyn TypstCompiler) -> Result<BuildOutput> {
    let root = root.as_ref();
    let paths = ProjectPaths::new(root);
    let input = load_manifest(root)?;

    recreate_dir(&paths.public)?;
    fs::create_dir_all(&paths.cache).at(&paths.cache)?;

    for article in input.public_articles() {
        build_article(&paths, &input, article, typst)?;
    }

    copy_dir_contents(&paths.site_src.join("static"), &paths.public.join("assets"))?;
    copy_dir_contents(
        &paths.typst_src.join("common/assets"),
        &paths.public.join("assets/common"),
    )?;

    let public_articles = input.public_articles();
    write_file(
        &paths.public.join("index.html"),
        render_index(&input.site, public_articles.clone()).map_err(|err| {
            BlogError::TypstFailed {
                article_dir: "<render>".to_owned(),
                entry: paths.public.join("index.html"),
                kind: "html",
                stderr: err.to_string(),
            }
        })?,
    )?;
    write_file(
        &paths.public.join("feed.xml"),
        render_feed(&input, &public_articles),
    )?;
    write_file(
        &paths.public.join("sitemap.xml"),
        render_sitemap(&input, &public_articles),
    )?;

    Ok(BuildOutput {
        public_dir: paths.public,
        cache_dir: paths.cache,
    })
}

fn build_article(
    paths: &ProjectPaths,
    input: &BuildInput,
    article: &ArticleManifest,
    typst: &dyn TypstCompiler,
) -> Result<()> {
    let cache_article_dir = paths.cache.join("articles").join(&article.dir_name);
    fs::create_dir_all(&cache_article_dir).at(&cache_article_dir)?;

    let body_path = cache_article_dir.join("body.html");
    let main_typ = article.dir_path.join("main.typ");
    typst
        .compile_html(&paths.typst_src, &main_typ, &body_path)
        .map_err(|err| typst_error(article, &main_typ, "html", err))?;
    write_file(&cache_article_dir.join("diagnostics.json"), "{}\n")?;

    let public_article_dir = paths.public.join("articles").join(&article.config.slug);
    fs::create_dir_all(&public_article_dir).at(&public_article_dir)?;

    let ogp_path = public_article_dir.join("ogp.png");
    let ogp_typ = article.dir_path.join("ogp.typ");
    typst
        .compile_png(&paths.typst_src, &ogp_typ, &ogp_path)
        .map_err(|err| typst_error(article, &ogp_typ, "ogp", err))?;

    copy_dir_contents(
        &article.dir_path.join("assets"),
        &public_article_dir.join("assets"),
    )?;

    let body_html = fs::read_to_string(&body_path).at(&body_path)?;
    let canonical_url = article_url(&input.site, article);
    let html = render_article(ArticlePage {
        site: &input.site,
        article,
        body_html: &body_html,
        ogp_url: format!("{canonical_url}ogp.png"),
        canonical_url,
    })
    .map_err(|err| BlogError::TypstFailed {
        article_dir: article.dir_name.clone(),
        entry: public_article_dir.join("index.html"),
        kind: "html",
        stderr: err.to_string(),
    })?;
    write_file(&public_article_dir.join("index.html"), html)
}

fn typst_error(
    article: &ArticleManifest,
    entry: &Path,
    kind: &'static str,
    err: BlogError,
) -> BlogError {
    BlogError::TypstFailed {
        article_dir: article.dir_name.clone(),
        entry: entry.to_path_buf(),
        kind,
        stderr: err.to_string(),
    }
}

fn render_feed(input: &BuildInput, articles: &[&ArticleManifest]) -> String {
    let items = articles
        .iter()
        .map(|article| {
            let url = article_url(&input.site, article);
            format!(
                "<item><title>{}</title><link>{}</link><guid>{}</guid><description>{}</description><pubDate>{}</pubDate></item>",
                escape_xml(&article.config.title),
                escape_xml(&url),
                escape_xml(&article.config.id.to_string()),
                escape_xml(&article.config.description),
                article.config.published_at.to_rfc2822()
            )
        })
        .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><rss version=\"2.0\"><channel><title>{}</title><link>{}</link><description>{}</description>{}</channel></rss>",
        escape_xml(&input.site.title),
        escape_xml(&input.site.base_url),
        escape_xml(&input.site.description),
        items
    )
}

fn render_sitemap(input: &BuildInput, articles: &[&ArticleManifest]) -> String {
    let urls = std::iter::once(format!(
        "<url><loc>{}/</loc></url>",
        escape_xml(input.site.base_url.trim_end_matches('/'))
    ))
    .chain(articles.iter().map(|article| {
        format!(
            "<url><loc>{}</loc><lastmod>{}</lastmod></url>",
            escape_xml(&article_url(&input.site, article)),
            article.config.updated_at.date_naive()
        )
    }))
    .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"
    )
}

fn recreate_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).at(path)?;
    }
    fs::create_dir_all(path).at(path)
}

fn write_file(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).at(parent)?;
    }
    fs::write(path, content).at(path)
}

fn copy_dir_contents(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).at(to)?;
    for entry in WalkDir::new(from)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let source = entry.path();
        let relative = source.strip_prefix(from).unwrap_or(source);
        let target = to.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).at(&target)?;
        } else {
            fs::copy(source, &target).at(&target)?;
        }
    }
    Ok(())
}
