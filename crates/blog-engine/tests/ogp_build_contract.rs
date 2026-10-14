use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use blog_engine::error::BlogEngineError;
use blog_engine::ogp::{
    build_ogp_public_path, generate_ogp_content_hash, OgpArtifact, OgpImageMeta, OgpRenderInput,
    OgpRenderSettings,
};
use blog_engine::ogp_render::OgpRenderer;
use blog_engine::typst_compile::TypstRunner;
use blog_engine::{build_with_runners, BuildReport};
use serde_json::Value;
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
            "<!DOCTYPE html><html><body><p>Generated body</p></body></html>",
        )
        .expect("fake Typst runner should write HTML");
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct Calls(Arc<Mutex<Vec<Call>>>);

#[derive(Debug, Clone)]
struct Call {
    input: OgpRenderInput,
    output_path: PathBuf,
}

#[derive(Debug, Clone)]
struct FakeOgpRenderer {
    calls: Calls,
    hash: OgpRenderSettings,
    fixed_hash: Option<String>,
}

impl FakeOgpRenderer {
    fn new() -> Self {
        Self {
            calls: Calls::default(),
            hash: OgpRenderSettings::new("fake-ogp-v1", "fake-template", 1200, 630, 72),
            fixed_hash: None,
        }
    }

    fn with_fixed_hash(mut self, hash: &str) -> Self {
        self.fixed_hash = Some(hash.to_owned());
        self
    }

    fn calls(&self) -> Vec<Call> {
        self.calls
            .0
            .lock()
            .expect("calls mutex should not be poisoned")
            .clone()
    }
}

impl OgpRenderer for FakeOgpRenderer {
    fn content_hash(
        &self,
        input: &OgpRenderInput,
        _output_path: &Path,
    ) -> Result<Option<String>, BlogEngineError> {
        Ok(Some(self.fixed_hash.clone().unwrap_or_else(|| {
            generate_ogp_content_hash(input, &self.hash)
        })))
    }

    fn render(
        &self,
        input: &OgpRenderInput,
        output_path: &Path,
    ) -> Result<OgpArtifact, BlogEngineError> {
        self.calls
            .0
            .lock()
            .expect("calls mutex should not be poisoned")
            .push(Call {
                input: input.clone(),
                output_path: output_path.to_path_buf(),
            });

        let bytes = fake_png();
        fs::write(output_path, &bytes).expect("fake OGP renderer should write PNG");
        let content_hash = self
            .content_hash(input, output_path)
            .expect("fake hash should be generated")
            .expect("fake hash should be planned");

        Ok(OgpArtifact {
            file_path: output_path.to_path_buf(),
            image: OgpImageMeta {
                path: build_ogp_public_path("unused", &content_hash),
                width: 1200,
                height: 630,
                mime_type: "image/png".to_owned(),
                alt: format!("{} | {}", input.article_title, input.site_title),
            },
            content_hash,
            byte_size: bytes.len() as u64,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct FailingOgpRenderer;

impl OgpRenderer for FailingOgpRenderer {
    fn content_hash(
        &self,
        _input: &OgpRenderInput,
        _output_path: &Path,
    ) -> Result<Option<String>, BlogEngineError> {
        Ok(Some("failure-hash".to_owned()))
    }

    fn render(
        &self,
        _input: &OgpRenderInput,
        _output_path: &Path,
    ) -> Result<OgpArtifact, BlogEngineError> {
        Err(BlogEngineError::Validation {
            message: "synthetic OGP failure".to_owned(),
        })
    }
}

#[test]
fn renders_each_unique_article_with_site_and_article_input() {
    let output = TempDir::new().expect("output directory should be created");
    let renderer = FakeOgpRenderer::new();

    let report = build_with_runners(
        fixture("valid-multiple"),
        output.path(),
        &FakeTypstRunner,
        &renderer,
    )
    .expect("build should succeed");

    let calls = renderer.calls();
    assert_eq!(calls.len(), 3);
    assert!(calls
        .iter()
        .all(|call| call.input.site_title == "neru7 blog"));
    assert_eq!(
        calls
            .iter()
            .map(|call| call.input.article_title.as_str())
            .collect::<Vec<_>>(),
        vec!["Newer Article", "Hello Typst", "Same Date A"]
    );
    assert!(calls.iter().all(|call| {
        call.output_path
            .parent()
            .expect("OGP path should have a parent")
            .ends_with("public-root/ogp/articles")
    }));
    assert!(calls.iter().all(|call| {
        let file_name = call
            .output_path
            .file_name()
            .expect("OGP path should have a file name")
            .to_string_lossy();
        file_name.ends_with(".png") && file_name.split('-').count() >= 3
    }));
    assert_eq!(report.ogp.images_generated, 3);
}

#[test]
fn successful_ogp_generation_is_registered_as_structured_metadata() {
    let output = TempDir::new().expect("output directory should be created");
    let renderer = FakeOgpRenderer::new();

    build_with_runners(
        fixture("valid-single"),
        output.path(),
        &FakeTypstRunner,
        &renderer,
    )
    .expect("build should succeed");

    let index = read_json(output.path().join("article-index.json"));
    let meta = read_json(output.path().join("articles/hello-typst/meta.json"));
    assert_eq!(index[0]["ogpImage"], meta["ogpImage"]);
    assert_eq!(index[0]["ogpImage"]["width"], 1200);
    assert_eq!(index[0]["ogpImage"]["height"], 630);
    assert_eq!(index[0]["ogpImage"]["mimeType"], "image/png");
    assert!(index[0]["ogpImage"]["path"]
        .as_str()
        .expect("OGP path should be a string")
        .starts_with("/ogp/articles/hello-typst-"));

    let public_path = index[0]["ogpImage"]["path"]
        .as_str()
        .expect("OGP path should be a string")
        .trim_start_matches('/');
    assert!(output
        .path()
        .join("public-root")
        .join(public_path)
        .is_file());
}

#[test]
fn ogp_failure_preserves_previous_output_and_writes_no_new_json() {
    let output = TempDir::new().expect("output directory should be created");
    let old_index = "old article index";
    let old_png = b"old png";
    fs::create_dir_all(output.path().join("public-root/ogp/articles"))
        .expect("old OGP directory should be created");
    fs::write(output.path().join("article-index.json"), old_index)
        .expect("old JSON should be written");
    fs::write(
        output.path().join("public-root/ogp/articles/old-hash.png"),
        old_png,
    )
    .expect("old PNG should be written");

    let error = build_with_runners(
        fixture("valid-single"),
        output.path(),
        &FakeTypstRunner,
        &FailingOgpRenderer,
    )
    .expect_err("OGP failure should fail the build");
    assert!(matches!(error, BlogEngineError::Validation { .. }));
    assert_eq!(
        fs::read_to_string(output.path().join("article-index.json"))
            .expect("old JSON should remain"),
        old_index
    );
    assert_eq!(
        fs::read(output.path().join("public-root/ogp/articles/old-hash.png"))
            .expect("old PNG should remain"),
        old_png
    );
}

#[test]
fn successful_build_removes_old_hashes_and_aggregates_statistics() {
    let output = TempDir::new().expect("output directory should be created");
    let old_path = output.path().join("public-root/ogp/articles/old-hash.png");
    fs::create_dir_all(
        old_path
            .parent()
            .expect("old OGP path should have a parent"),
    )
    .expect("old OGP directory should be created");
    fs::write(&old_path, b"old png").expect("old PNG should be written");
    let renderer = FakeOgpRenderer::new();

    let report: BuildReport = build_with_runners(
        fixture("valid-multiple"),
        output.path(),
        &FakeTypstRunner,
        &renderer,
    )
    .expect("build should succeed");
    let fake_size = fake_png().len() as u64;

    assert!(!old_path.exists());
    assert_eq!(report.ogp.images_generated, 3);
    assert_eq!(report.ogp.total_size, fake_size * 3);
    assert_eq!(report.ogp.largest_file, fake_size);
}

#[test]
fn identical_content_hash_is_rendered_once_and_reused_per_article_path() {
    let output = TempDir::new().expect("output directory should be created");
    let renderer = FakeOgpRenderer::new()
        .with_fixed_hash("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

    build_with_runners(
        fixture("valid-multiple"),
        output.path(),
        &FakeTypstRunner,
        &renderer,
    )
    .expect("build should succeed");

    assert_eq!(renderer.calls().len(), 1);
    let ogp_dir = output.path().join("public-root/ogp/articles");
    let mut files = fs::read_dir(ogp_dir)
        .expect("OGP directory should exist")
        .map(|entry| entry.expect("OGP entry should be readable").file_name())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), 3);
    assert!(files
        .iter()
        .all(|file| file.to_string_lossy().ends_with("-0123456789ab.png")));
}

fn fake_png() -> Vec<u8> {
    let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
    bytes.extend_from_slice(&13_u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&1200_u32.to_be_bytes());
    bytes.extend_from_slice(&630_u32.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("JSON should exist"))
        .expect("JSON should be valid")
}
