use std::collections::HashMap;
use std::path::PathBuf;

use lol_html::{element, html_content::ContentType, rewrite_str, Settings};

use super::cache;
use super::metadata::CardMetadata;
use super::resolver::{parse_requested_url, MetadataResolver};
use crate::error::BlogEngineError;

/// なぜ実装するのか:
/// - リンクカードprocessorの依存を具体的かつテスト可能な形で保持するため。
///
/// 期待する動作:
/// - cache rootとメタデータresolverだけを提供する。
///
/// 使い方の概要:
/// - `PostprocessContext` または単体テストで構築する。
pub struct WebLinkCardContext {
    pub(crate) cache_root: PathBuf,
    pub(crate) resolver: Box<dyn MetadataResolver>,
}

impl WebLinkCardContext {
    pub fn new(cache_root: PathBuf, resolver: Box<dyn MetadataResolver>) -> Self {
        Self {
            cache_root,
            resolver,
        }
    }
}

/// なぜ実装するのか:
/// - 明示的なmarkerを持つanchorだけを安全なカードHTMLへ変換するため。
///
/// 期待する動作:
/// - 外部取得失敗ではfallbackを返し、markerは常に最終HTMLから除く。
///
/// 使い方の概要:
/// - 本文fragmentと文脈を渡す。
pub fn process(html: String, context: &WebLinkCardContext) -> Result<String, BlogEngineError> {
    if !html.contains("web-link-card-flag=\"true\"") {
        return Ok(html);
    }
    let mut resolved = HashMap::<String, Result<CardMetadata, String>>::new();
    let output = rewrite_str(
        &html,
        Settings {
            element_content_handlers: vec![element!("a[web-link-card-flag=\"true\"]", |element| {
                let href = element.get_attribute("href").unwrap_or_default();
                let replacement = match resolve_once(&href, context, &mut resolved) {
                    Ok(metadata) => render_card(&href, metadata),
                    Err(error) => {
                        eprintln!("warning: web-link-card fallback for {href}: {error}");
                        render_fallback(&href)
                    }
                };
                element.replace(&replacement, ContentType::Html);
                Ok(())
            })],
            ..Settings::default()
        },
    );
    output.map_err(|error| BlogEngineError::html_postprocess(error.to_string()))
}

fn resolve_once<'a>(
    href: &str,
    context: &'a WebLinkCardContext,
    resolved: &'a mut HashMap<String, Result<CardMetadata, String>>,
) -> &'a Result<CardMetadata, String> {
    resolved.entry(href.to_owned()).or_insert_with(|| {
        let url = parse_requested_url(href)?;
        let normalized = url.to_string();
        if let Some(metadata) = cache::load(&context.cache_root, &normalized) {
            return Ok(metadata);
        }
        let metadata = context.resolver.resolve(&url)?;
        cache::store(&context.cache_root, &normalized, &metadata);
        Ok(metadata)
    })
}

fn escaped(value: &str) -> String {
    html_escape(value)
}
fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn render_card(href: &str, metadata: &CardMetadata) -> String {
    let image = metadata
        .image_url
        .as_ref()
        .map(|image| {
            format!(
                "<img class=\"web-link-card__image\" src=\"{}\" alt=\"\" loading=\"lazy\">",
                escaped(image)
            )
        })
        .unwrap_or_default();
    let description = metadata
        .description
        .as_ref()
        .map(|description| {
            format!(
                "<span class=\"web-link-card__description\">{}</span>",
                escaped(description)
            )
        })
        .unwrap_or_default();
    format!("<a href=\"{}\" class=\"web-link-card\">{image}<span class=\"web-link-card__content\"><span class=\"web-link-card__title\">{}</span>{description}<span class=\"web-link-card__site\">{}</span></span></a>", escaped(href), escaped(&metadata.title), escaped(&metadata.site_name))
}
fn render_fallback(href: &str) -> String {
    format!("<a href=\"{}\" class=\"web-link-card web-link-card--fallback\"><span class=\"web-link-card__fallback\">{}</span></a>", escaped(href), escaped(href))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;
    use url::Url;
    struct Fake {
        calls: Arc<AtomicUsize>,
        result: Result<CardMetadata, String>,
    }
    impl MetadataResolver for Fake {
        fn resolve(&self, _: &Url) -> Result<CardMetadata, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result.clone()
        }
    }
    fn metadata() -> CardMetadata {
        CardMetadata {
            schema_version: 1,
            requested_url: "https://example.com/one".into(),
            resolved_url: "https://example.com/one".into(),
            title: "A <title>".into(),
            description: Some("description & more".into()),
            site_name: "Example".into(),
            image_url: Some("https://example.com/image.png".into()),
            fetched_at: "2020-01-01T00:00:00Z".into(),
        }
    }
    fn context(
        result: Result<CardMetadata, String>,
    ) -> (WebLinkCardContext, Arc<AtomicUsize>, TempDir) {
        let dir = TempDir::new().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        (
            WebLinkCardContext::new(
                dir.path().to_owned(),
                Box::new(Fake {
                    calls: calls.clone(),
                    result,
                }),
            ),
            calls,
            dir,
        )
    }
    #[test]
    fn unmarked_html_is_unchanged() {
        let (ctx, _, _) = context(Ok(metadata()));
        assert_eq!(process("<p>x</p>".into(), &ctx).unwrap(), "<p>x</p>");
    }
    #[test]
    fn only_exact_marked_anchor_is_processed() {
        let (ctx, calls, _) = context(Ok(metadata()));
        let output=process("<a web-link-card-flag=\"false\" href=\"https://example.com/one\">x</a><div web-link-card-flag=\"true\">y</div><a web-link-card-flag=\"true\" href=\"https://example.com/one\">x</a>".into(),&ctx).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(output.contains("web-link-card-flag=\"false\""));
        assert!(output.contains("<div web-link-card-flag=\"true\""));
        assert!(!output.contains("<a web-link-card-flag=\"true\""));
    }
    #[test]
    fn resolved_card_renders_and_escapes() {
        let (ctx, _, _) = context(Ok(metadata()));
        let output = process(
            "<a web-link-card-flag=\"true\" href=\"https://example.com/one\">x</a>".into(),
            &ctx,
        )
        .unwrap();
        assert!(output.contains("href=\"https://example.com/one\""));
        assert!(output.contains("web-link-card__image"));
        assert!(output.contains("A &lt;title&gt;"));
        assert!(output.contains("description &amp; more"));
        assert!(!output.contains("web-link-card-flag"));
    }
    #[test]
    fn failure_keeps_clickable_fallback() {
        let (ctx, _, _) = context(Err("offline".into()));
        let output = process(
            "<a web-link-card-flag=\"true\" href=\"https://example.com/one\">x</a>".into(),
            &ctx,
        )
        .unwrap();
        assert!(output.contains("web-link-card--fallback"));
        assert!(output.contains("href=\"https://example.com/one\""));
        assert!(!output.contains("web-link-card-flag"));
    }
    #[test]
    fn duplicate_urls_are_resolved_once_and_cache_hit_skips_resolver() {
        let (ctx, calls, _) = context(Ok(metadata()));
        let html="<a web-link-card-flag=\"true\" href=\"https://example.com/one\">x</a><a web-link-card-flag=\"true\" href=\"https://example.com/one\">x</a>";
        process(html.into(), &ctx).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        process(html.into(), &ctx).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
