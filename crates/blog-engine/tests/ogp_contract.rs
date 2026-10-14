use std::path::PathBuf;

use blog_engine::article::Article;
use blog_engine::ogp::{
    build_ogp_public_path, generate_ogp_content_hash, OgpArtifact, OgpImageMeta, OgpRenderInput,
    OgpRenderSettings,
};
use blog_engine::output::SiteOutput;
use chrono::{DateTime, FixedOffset};

fn site() -> SiteOutput {
    SiteOutput {
        title: "  Example   Blog  ".to_owned(),
        description: "A   site description".to_owned(),
        language: "ja".to_owned(),
    }
}

fn article() -> Article {
    Article {
        id: "article-1".to_owned(),
        slug: "sample-typst".to_owned(),
        title: "  Example   Article  ".to_owned(),
        description: "An   article description".to_owned(),
        published_at: "2025-01-01T00:00:00+09:00"
            .parse::<DateTime<FixedOffset>>()
            .expect("fixture date must be valid"),
        updated_at: None,
        tags: vec![" Rust ".to_owned(), "".to_owned(), "Rust".to_owned()],
        url: "/articles/sample-typst/".to_owned(),
    }
}

fn render_input() -> OgpRenderInput {
    OgpRenderInput::from_models(&site(), &article())
}

fn render_settings() -> OgpRenderSettings {
    OgpRenderSettings::new(
        "ogp-renderer-v1",
        "#set page(width: 1200pt, height: 630pt)\n#let render() = []",
        1200,
        630,
        144,
    )
}

#[test]
fn render_input_is_built_from_validated_models() {
    let actual = render_input();

    assert_eq!(actual.site_title, "Example Blog");
    assert_eq!(actual.article_title, "Example Article");
    assert_eq!(actual.description, "An article description");
    assert_eq!(actual.tags, vec!["Rust"]);
}

#[test]
fn building_render_input_does_not_mutate_source_models() {
    let site = site();
    let article = article();
    let original_site = site.clone();
    let original_article = article.clone();

    let _ = OgpRenderInput::from_models(&site, &article);

    assert_eq!(site, original_site);
    assert_eq!(article, original_article);
}

#[test]
fn identical_input_produces_identical_hash() {
    let input = render_input();
    let settings = render_settings();

    assert_eq!(
        generate_ogp_content_hash(&input, &settings),
        generate_ogp_content_hash(&input, &settings)
    );
}

#[test]
fn hash_is_a_lowercase_sha256_string() {
    let hash = generate_ogp_content_hash(&render_input(), &render_settings());

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|character| character.is_ascii_hexdigit()));
    assert_eq!(hash, hash.to_ascii_lowercase());
}

#[test]
fn changing_site_title_changes_hash() {
    let mut input = render_input();
    let settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    input.site_title.push('!');

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_title_changes_hash() {
    let mut input = render_input();
    let settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    input.article_title.push('!');

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_description_changes_hash() {
    let mut input = render_input();
    let settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    input.description.push('!');

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_tag_content_changes_hash() {
    let mut input = render_input();
    let settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    input.tags[0] = "Typst".to_owned();

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_tag_order_changes_hash() {
    let mut input = render_input();
    input.tags.push("Typst".to_owned());
    let settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    input.tags.swap(0, 1);

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_template_changes_hash() {
    let input = render_input();
    let mut settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    settings.template.push_str("\n#let version = 2");

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_width_changes_hash() {
    let input = render_input();
    let mut settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    settings.width += 1;

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_height_changes_hash() {
    let input = render_input();
    let mut settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    settings.height += 1;

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_ppi_changes_hash() {
    let input = render_input();
    let mut settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    settings.ppi += 1;

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn changing_renderer_version_changes_hash() {
    let input = render_input();
    let mut settings = render_settings();
    let original = generate_ogp_content_hash(&input, &settings);

    settings.renderer_version.push_str("-next");

    assert_ne!(original, generate_ogp_content_hash(&input, &settings));
}

#[test]
fn public_path_uses_the_hash_prefix() {
    let input = render_input();
    let hash = generate_ogp_content_hash(&input, &render_settings());

    assert_eq!(
        build_ogp_public_path("sample-typst", &hash),
        format!("/ogp/articles/sample-typst-{}.png", &hash[..12])
    );
}

#[test]
fn public_path_does_not_use_os_path_separators() {
    let path = build_ogp_public_path("sample-typst", "a82c19d843f1deadbeef");

    assert!(!path.contains('\\'));
}

#[test]
fn artifact_distinguishes_filesystem_path_from_public_metadata() {
    let image = OgpImageMeta {
        path: "/ogp/articles/sample-typst-a82c19d843f1.png".to_owned(),
        width: 1200,
        height: 630,
        mime_type: "image/png".to_owned(),
        alt: "Example Article | Example Blog".to_owned(),
    };
    let artifact = OgpArtifact {
        file_path: PathBuf::from(
            "generated/public-root/ogp/articles/sample-typst-a82c19d843f1.png",
        ),
        image,
        content_hash: "a82c19d843f1deadbeef".to_owned(),
        byte_size: 42,
    };

    assert_eq!(
        artifact.file_path,
        PathBuf::from("generated/public-root/ogp/articles/sample-typst-a82c19d843f1.png")
    );
    assert_eq!(
        artifact.image.path,
        "/ogp/articles/sample-typst-a82c19d843f1.png"
    );
}

#[test]
fn image_metadata_uses_camel_case_json_keys() {
    let image = OgpImageMeta {
        path: "/ogp/articles/sample-typst-a82c19d843f1.png".to_owned(),
        width: 1200,
        height: 630,
        mime_type: "image/png".to_owned(),
        alt: "Example Article | Example Blog".to_owned(),
    };
    let value = serde_json::to_value(image).expect("image metadata must serialize");

    assert_eq!(value["mimeType"], "image/png");
    assert!(value.get("mime_type").is_none());
}
