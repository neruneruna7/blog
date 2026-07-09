use std::fs;
use std::path::Path;

use blog_proto::build::build_site;
use blog_proto::check::check_site;
use blog_proto::error::{BlogError, Result};
use blog_proto::manifest::load_manifest;
use blog_proto::new_article::create_article;
use blog_proto::typst::TypstCompiler;
use tempfile::TempDir;

struct FakeTypst {
    fail_html: bool,
}

impl TypstCompiler for FakeTypst {
    fn compile_html(&self, _root: &Path, input: &Path, output: &Path) -> Result<()> {
        if self.fail_html {
            return Err(BlogError::TypstFailed {
                article_dir: "fake".to_owned(),
                entry: input.to_path_buf(),
                kind: "html",
                stderr: "fake typst failure".to_owned(),
            });
        }
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, "<p>compiled body</p>").unwrap();
        Ok(())
    }

    fn compile_png(&self, _root: &Path, _input: &Path, output: &Path) -> Result<()> {
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, b"png").unwrap();
        Ok(())
    }
}

#[test]
fn valid_article_config_parses() {
    let dir = fixture();

    let input = load_manifest(dir.path()).unwrap();

    assert_eq!(input.articles.len(), 1);
    assert_eq!(input.articles[0].config.slug, "public-slug");
}

#[test]
fn missing_title_fails() {
    let dir = fixture();
    let config = article_config("0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f5", "public-slug", false)
        .replace("title = \"Example\"\n", "");
    fs::write(
        dir.path()
            .join("typst-src/articles/my-local-name/ArticleConfig.toml"),
        config,
    )
    .unwrap();

    assert!(load_manifest(dir.path()).is_err());
}

#[test]
fn invalid_published_at_fails() {
    let dir = fixture();
    let config = article_config("0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f5", "public-slug", false)
        .replace(
            "published_at = \"2026-07-08T00:00:00+09:00\"",
            "published_at = \"not-a-date\"",
        );
    fs::write(
        dir.path()
            .join("typst-src/articles/my-local-name/ArticleConfig.toml"),
        config,
    )
    .unwrap();

    assert!(load_manifest(dir.path()).is_err());
}

#[test]
fn duplicate_id_fails() {
    let dir = fixture();
    add_article(
        dir.path(),
        "other-local-name",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f5",
        "other-slug",
        false,
    );

    assert!(matches!(
        load_manifest(dir.path()),
        Err(BlogError::DuplicateId { .. })
    ));
}

#[test]
fn duplicate_slug_fails() {
    let dir = fixture();
    add_article(
        dir.path(),
        "other-local-name",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f6",
        "public-slug",
        false,
    );

    assert!(matches!(
        load_manifest(dir.path()),
        Err(BlogError::DuplicateSlug { .. })
    ));
}

#[test]
fn draft_is_excluded_from_public_manifest() {
    let dir = fixture();
    add_article(
        dir.path(),
        "draft-local",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f6",
        "draft-slug",
        true,
    );

    let input = load_manifest(dir.path()).unwrap();

    assert_eq!(input.articles.len(), 2);
    assert_eq!(input.public_articles().len(), 1);
    assert_eq!(input.public_articles()[0].config.slug, "public-slug");
}

#[test]
fn build_outputs_use_slug_not_article_directory() {
    let dir = fixture();

    build_site(dir.path(), &FakeTypst { fail_html: false }).unwrap();

    assert!(dir.path().join("public/index.html").exists());
    assert!(
        dir.path()
            .join("public/articles/public-slug/index.html")
            .exists()
    );
    assert!(
        dir.path()
            .join("public/articles/public-slug/ogp.png")
            .exists()
    );
    assert!(
        dir.path()
            .join("public/articles/public-slug/assets/image.txt")
            .exists()
    );
    assert!(dir.path().join("public/assets/main.css").exists());
    assert!(dir.path().join("public/assets/common/common.txt").exists());
    assert!(!dir.path().join("public/articles/my-local-name").exists());
    assert!(
        !dir.path()
            .join("public/articles/public-slug/body.html")
            .exists()
    );
}

#[test]
fn build_excludes_drafts_from_public_outputs() {
    let dir = fixture();
    add_article(
        dir.path(),
        "draft-local",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f6",
        "draft-slug",
        true,
    );

    build_site(dir.path(), &FakeTypst { fail_html: false }).unwrap();

    assert!(!dir.path().join("public/articles/draft-slug").exists());
    let index = fs::read_to_string(dir.path().join("public/index.html")).unwrap();
    assert!(!index.contains("draft-slug"));
}

#[test]
fn build_fails_when_typst_fails() {
    let dir = fixture();

    let err = build_site(dir.path(), &FakeTypst { fail_html: true }).unwrap_err();

    assert!(matches!(err, BlogError::TypstFailed { kind: "html", .. }));
    assert!(err.to_string().contains("my-local-name"));
}

#[test]
fn check_validates_without_creating_public() {
    let dir = fixture();

    check_site(dir.path()).unwrap();

    assert!(!dir.path().join("public").exists());
}

#[test]
fn check_reports_duplicate_slug() {
    let dir = fixture();
    add_article(
        dir.path(),
        "other-local-name",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f6",
        "public-slug",
        false,
    );

    assert!(matches!(
        check_site(dir.path()),
        Err(BlogError::DuplicateSlug { .. })
    ));
}

#[test]
fn new_creates_article_skeleton() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("typst-src/articles")).unwrap();

    create_article(dir.path(), "Hello World").unwrap();

    let article_dir = dir.path().join("typst-src/articles/hello-world");
    assert!(article_dir.join("ArticleConfig.toml").exists());
    assert!(article_dir.join("main.typ").exists());
    assert!(article_dir.join("ogp.typ").exists());
    assert!(article_dir.join("assets").is_dir());

    let config = fs::read_to_string(article_dir.join("ArticleConfig.toml")).unwrap();
    assert!(config.contains("slug = \"hello-world\""));
    let id_line = config
        .lines()
        .find(|line| line.starts_with("id = "))
        .unwrap();
    let id = id_line.trim_start_matches("id = \"").trim_end_matches('"');
    assert_eq!(id.as_bytes()[14] as char, '7');
}

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("typst-src/articles")).unwrap();
    fs::create_dir_all(dir.path().join("typst-src/common/assets")).unwrap();
    fs::create_dir_all(dir.path().join("site-src/static")).unwrap();
    fs::write(
        dir.path().join("typst-src/SiteConfig.toml"),
        r#"title = "neru7.log"
description = "Personal engineering blog."
base_url = "https://example.com"
author = "neru7"
language = "ja-JP"
timezone = "Asia/Tokyo"
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("typst-src/common/assets/common.txt"),
        "common",
    )
    .unwrap();
    fs::write(dir.path().join("site-src/static/main.css"), "body{}").unwrap();
    add_article(
        dir.path(),
        "my-local-name",
        "0192f4e2-7c4a-7a8b-9d8e-a0b1c2d3e4f5",
        "public-slug",
        false,
    );
    dir
}

fn add_article(root: &Path, dir_name: &str, id: &str, slug: &str, draft: bool) {
    let article_dir = root.join("typst-src/articles").join(dir_name);
    fs::create_dir_all(article_dir.join("assets")).unwrap();
    fs::write(
        article_dir.join("ArticleConfig.toml"),
        article_config(id, slug, draft),
    )
    .unwrap();
    fs::write(article_dir.join("main.typ"), "= Example\n").unwrap();
    fs::write(
        article_dir.join("ogp.typ"),
        "#set page(width: 1200pt, height: 630pt)\n= OGP\n",
    )
    .unwrap();
    fs::write(article_dir.join("assets/image.txt"), "image").unwrap();
}

fn article_config(id: &str, slug: &str, draft: bool) -> String {
    format!(
        r#"id = "{id}"
slug = "{slug}"
title = "Example"
description = "A prototype static blog engine using Rust and Typst."
tags = ["rust", "typst", "blog"]
published_at = "2026-07-08T00:00:00+09:00"
updated_at = "2026-07-08T00:00:00+09:00"
draft = {draft}
"#
    )
}
