use std::fs;
use std::path::{Path, PathBuf};

use blog_engine::article::ArticleMeta;
use blog_engine::config::load_site_config;
use blog_engine::error::BlogEngineError;
use blog_engine::typst_compile::TypstRunner;
use blog_engine::{build, build_with_typst_runner, load_normalized_articles};
use serde_json::{json, Value};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
        .join("typst-src")
}

#[derive(Debug, Clone, Copy)]
struct FakeTypstRunner;

impl TypstRunner for FakeTypstRunner {
    fn compile(
        &self,
        _slug: &str,
        _input_path: &Path,
        output_path: &Path,
    ) -> Result<(), BlogEngineError> {
        fs::write(
            output_path,
            "<!DOCTYPE html><html><head></head><body><h2>Hello</h2><p>Generated body</p></body></html>",
        )
        .expect("fake runner should write HTML");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct FailingTypstRunner;

impl TypstRunner for FailingTypstRunner {
    fn compile(
        &self,
        slug: &str,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), BlogEngineError> {
        Err(BlogEngineError::typst_compile_error(
            slug,
            input_path,
            output_path,
            "synthetic typst failure",
        ))
    }
}

#[test]
fn valid_site_toml_is_normalized_into_site_json_shape() {
    let site = load_site_config(fixture("valid-single")).expect("site config should be valid");
    let actual = serde_json::to_value(site).expect("site output should serialize");

    assert_eq!(
        actual,
        json!({
            "title": "neru7 blog",
            "description": "Typst-based technical blog",
            "language": "ja",
            "baseUrl": "http://localhost:4321"
        })
    );
}

#[test]
fn valid_article_config_toml_is_normalized_into_article_metadata_shape() {
    let article = single_article("valid-single");
    let actual = serde_json::to_value(article).expect("article output should serialize");

    assert_eq!(
        actual,
        json!({
            "id": "01JZQ2EXAMPLE0000000000000",
            "slug": "hello-typst",
            "title": "Hello Typst",
            "description": "Typstでブログ記事を書く最小例",
            "publishedAt": "2026-07-09T00:00:00+09:00",
            "updatedAt": null,
            "tags": ["typst", "rust"],
            "url": "/articles/hello-typst/",
            "ogpImage": null
        })
    );
}

#[test]
fn empty_updated_at_becomes_null() {
    assert_eq!(single_article("valid-single").updated_at, None);
}

#[test]
fn article_url_is_generated_from_slug() {
    assert_eq!(single_article("valid-single").url, "/articles/hello-typst/");
}

#[test]
fn article_index_is_sorted_by_published_at_descending() {
    let articles =
        load_normalized_articles(fixture("valid-multiple")).expect("articles should be valid");

    assert_eq!(
        articles
            .iter()
            .map(|article| article.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["newer-article", "hello-typst", "same-date-a"]
    );
}

#[test]
fn duplicate_slug_is_rejected() {
    assert_validation_error("duplicate-slug", "slug must be unique");
}

#[test]
fn duplicate_id_is_rejected() {
    assert_validation_error("duplicate-id", "id must be unique");
}

#[test]
fn missing_required_site_field_is_rejected() {
    let error =
        load_site_config(fixture("missing-site-field")).expect_err("site should be rejected");

    assert!(matches!(error, BlogEngineError::Toml { .. }));
}

#[test]
fn missing_required_article_field_is_rejected() {
    let error = load_normalized_articles(fixture("missing-article-field"))
        .expect_err("article should be rejected");

    assert!(matches!(error, BlogEngineError::Toml { .. }));
}

#[test]
fn invalid_published_at_is_rejected() {
    assert_validation_error("invalid-published-at", "published_at must be RFC3339");
}

#[test]
fn invalid_updated_at_is_rejected() {
    assert_validation_error("invalid-updated-at", "updated_at must be RFC3339");
}

#[test]
fn invalid_slug_is_rejected() {
    assert_validation_error("invalid-slug", "slug is invalid");
}

#[test]
fn article_meta_json_is_written_under_slug_directory() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    let meta_path = output
        .path()
        .join("articles")
        .join("hello-typst")
        .join("meta.json");
    let actual = read_json(meta_path);
    assert_eq!(actual["slug"], "hello-typst");
    assert_eq!(actual["url"], "/articles/hello-typst/");
}

#[test]
fn generated_json_is_deterministic() {
    let first = TempDir::new().expect("temp dir should be created");
    let second = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-multiple"), first.path(), &FakeTypstRunner)
        .expect("first build should succeed");
    build_with_typst_runner(fixture("valid-multiple"), second.path(), &FakeTypstRunner)
        .expect("second build should succeed");

    assert_eq!(
        fs::read_to_string(first.path().join("article-index.json"))
            .expect("first index should exist"),
        fs::read_to_string(second.path().join("article-index.json"))
            .expect("second index should exist")
    );
}

#[test]
fn id_and_slug_remain_separate_fields() {
    let article = single_article("valid-single");

    assert_eq!(article.id, "01JZQ2EXAMPLE0000000000000");
    assert_eq!(article.slug, "hello-typst");
    assert_ne!(article.id, article.slug);
}

#[test]
fn build_writes_site_and_article_index_json() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    assert_eq!(
        read_json(output.path().join("site.json"))["baseUrl"],
        "http://localhost:4321"
    );
    assert_eq!(
        read_json(output.path().join("article-index.json"))[0]["slug"],
        "hello-typst"
    );
}

#[test]
fn build_writes_article_body_html() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    assert!(output
        .path()
        .join("articles")
        .join("hello-typst")
        .join("body.html")
        .is_file());
}

#[test]
fn body_html_is_not_empty() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    let body = read_text(
        output
            .path()
            .join("articles")
            .join("hello-typst")
            .join("body.html"),
    );
    assert!(!body.trim().is_empty());
}

#[test]
fn body_html_does_not_contain_document_level_tags() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    let body = read_text(
        output
            .path()
            .join("articles")
            .join("hello-typst")
            .join("body.html"),
    )
    .to_lowercase();
    assert!(!body.contains("<!doctype html"));
    assert!(!body.contains("<html"));
    assert!(!body.contains("<head"));
    assert!(!body.contains("<body"));
}

#[test]
fn missing_main_typ_is_rejected() {
    let output = TempDir::new().expect("temp dir should be created");
    let error =
        build_with_typst_runner(fixture("missing-main-typ"), output.path(), &FakeTypstRunner)
            .expect_err("missing main.typ should be rejected");

    assert!(matches!(error, BlogEngineError::MissingMainTyp { .. }));
}

#[test]
fn typst_compile_failure_is_rejected() {
    let output = TempDir::new().expect("temp dir should be created");
    let error =
        build_with_typst_runner(fixture("invalid-typst"), output.path(), &FailingTypstRunner)
            .expect_err("typst failure should be rejected");

    assert!(matches!(error, BlogEngineError::TypstCompile { .. }));
}

#[test]
fn existing_site_json_output_remains_unchanged() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    assert_eq!(
        read_json(output.path().join("site.json")),
        json!({
            "title": "neru7 blog",
            "description": "Typst-based technical blog",
            "language": "ja",
            "baseUrl": "http://localhost:4321"
        })
    );
}

#[test]
fn existing_article_index_json_output_remains_unchanged() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    assert_eq!(
        read_json(output.path().join("article-index.json")),
        json!([{
            "id": "01JZQ2EXAMPLE0000000000000",
            "slug": "hello-typst",
            "title": "Hello Typst",
            "description": "Typstでブログ記事を書く最小例",
            "publishedAt": "2026-07-09T00:00:00+09:00",
            "updatedAt": null,
            "tags": ["typst", "rust"],
            "url": "/articles/hello-typst/",
            "ogpImage": null
        }])
    );
}

#[test]
fn existing_meta_json_output_remains_unchanged() {
    let output = TempDir::new().expect("temp dir should be created");

    build_with_typst_runner(fixture("valid-single"), output.path(), &FakeTypstRunner)
        .expect("build should succeed");

    assert_eq!(
        read_json(
            output
                .path()
                .join("articles")
                .join("hello-typst")
                .join("meta.json")
        ),
        json!({
            "id": "01JZQ2EXAMPLE0000000000000",
            "slug": "hello-typst",
            "title": "Hello Typst",
            "description": "Typstでブログ記事を書く最小例",
            "publishedAt": "2026-07-09T00:00:00+09:00",
            "updatedAt": null,
            "tags": ["typst", "rust"],
            "url": "/articles/hello-typst/",
            "ogpImage": null
        })
    );
}

#[test]
fn real_typst_conversion_is_env_gated() {
    if std::env::var("BLOG_ENGINE_RUN_TYPST_INTEGRATION")
        .ok()
        .as_deref()
        != Some("1")
    {
        return;
    }

    let output = TempDir::new().expect("temp dir should be created");
    build(fixture("valid-single"), output.path()).expect("real typst build should succeed");

    let body = read_text(
        output
            .path()
            .join("articles")
            .join("hello-typst")
            .join("body.html"),
    );
    assert!(!body.trim().is_empty());
    assert!(!body.to_lowercase().contains("<html"));
}

fn single_article(fixture_name: &str) -> ArticleMeta {
    load_normalized_articles(fixture(fixture_name))
        .expect("articles should be valid")
        .into_iter()
        .next()
        .expect("fixture should contain one article")
}

fn assert_validation_error(fixture_name: &str, expected_message: &str) {
    let error =
        load_normalized_articles(fixture(fixture_name)).expect_err("fixture should be rejected");
    match error {
        BlogEngineError::Validation { message } => assert!(
            message.contains(expected_message),
            "expected message to contain {expected_message:?}, got {message:?}"
        ),
        other => panic!("expected validation error, got {other:?}"),
    }
}

fn read_json(path: impl AsRef<Path>) -> Value {
    let content = fs::read_to_string(path).expect("json file should exist");
    serde_json::from_str(&content).expect("json should be valid")
}

fn read_text(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("text file should exist")
}
