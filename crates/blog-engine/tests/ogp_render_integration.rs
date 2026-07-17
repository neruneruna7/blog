use std::fs;
use std::process::Command;

use blog_engine::ogp::OgpRenderInput;
use blog_engine::ogp_render::{OgpRenderer, TypstOgpRenderer, OGP_HEIGHT_PX, OGP_WIDTH_PX};
use tempfile::TempDir;

fn template_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../typst-src/commons/ogp/template.typ")
}

fn renderer() -> TypstOgpRenderer {
    TypstOgpRenderer::new(
        template_path(),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../typst-src"),
    )
}

fn input() -> OgpRenderInput {
    OgpRenderInput {
        site_title: "neru7 blog".to_owned(),
        article_title: "日本語のタイトルをTypstで描画する".to_owned(),
        description: "TypstでOGP画像を生成する統合テスト".to_owned(),
        tags: vec!["typst".to_owned(), "rust".to_owned()],
    }
}

const REQUIRED_OGP_FONTS: [&str; 6] = [
    "Geist",
    "Hina Mincho",
    "Kaisei Decol",
    "Noto Sans JP",
    "Noto Serif JP",
    "Shippori Mincho",
];

fn assert_ogp_fonts_are_available() {
    let output = Command::new("typst")
        .arg("fonts")
        .output()
        .expect("Typst integration requires the `typst` CLI");
    assert!(
        output.status.success(),
        "Typst integration requires `typst fonts` to list installed fonts"
    );
    let fonts = String::from_utf8_lossy(&output.stdout);
    let available_fonts = fonts.lines().map(str::trim).collect::<Vec<_>>();
    let missing_fonts = REQUIRED_OGP_FONTS
        .iter()
        .copied()
        .filter(|font| !available_fonts.contains(font))
        .collect::<Vec<_>>();
    assert!(
        missing_fonts.is_empty(),
        "Typst integration requires all OGP fonts; missing: {missing_fonts:?}"
    );
}

#[test]
fn template_renders_a_valid_japanese_png_at_the_contract_dimensions() {
    assert_ogp_fonts_are_available();
    let output = TempDir::new().expect("temporary output directory must be created");
    let output_path = output.path().join("article.png");

    let artifact = renderer().render(&input(), &output_path).expect(
        "Typst integration requires the `typst` CLI and a Japanese-capable font; rendering failed",
    );

    assert_eq!(artifact.file_path, output_path);
    assert_eq!(artifact.image.width, OGP_WIDTH_PX);
    assert_eq!(artifact.image.height, OGP_HEIGHT_PX);
    assert_eq!(artifact.image.mime_type, "image/png");
    assert!(artifact.byte_size > 0);
    assert!(fs::metadata(&output_path)
        .expect("rendered PNG must exist")
        .is_file());
}

#[test]
fn template_renders_a_long_japanese_title_without_failing() {
    assert_ogp_fonts_are_available();
    let output = TempDir::new().expect("temporary output directory must be created");
    let output_path = output.path().join("long-title.png");
    let input = OgpRenderInput {
        article_title: "これは非常に長い日本語タイトルでありOGP画像の固定された表示領域へ収まるように文字サイズを段階的に縮小しそれでも長すぎる場合には表示用の派生文字列だけを省略することを検証するための統合テスト用タイトルである".to_owned(),
        ..input()
    };

    let artifact = renderer().render(&input, &output_path).expect(
        "Typst integration requires the `typst` CLI and a Japanese-capable font; rendering failed",
    );

    assert_eq!(artifact.image.width, OGP_WIDTH_PX);
    assert_eq!(artifact.image.height, OGP_HEIGHT_PX);
    assert!(artifact.byte_size > 0);
}
