//! リンクカードのOGP情報をHTMLへ変換する。

use std::collections::HashSet;

use lol_html::{Settings, element, html_content::ContentType, rewrite_str};

use crate::features::{
    article_transform::{ArticleCompileWarning, error::Error},
    web_link_card_ogp_input::{WebLinkCardOgp, WebLinkCardOgpInput},
};

pub(super) struct Processor;

impl Processor {
    pub(super) fn process(
        &self,
        html: &str,
        cards: &WebLinkCardOgpInput,
        slug: &str,
    ) -> Result<Output, Error> {
        let (html, warnings) = transform(html, cards, slug)?;

        Ok(Output { html, warnings })
    }
}

pub(super) struct Output {
    pub(super) html: String,
    pub(super) warnings: Vec<ArticleCompileWarning>,
}

pub(super) fn collect_urls(html: &str, slug: &str) -> Result<Vec<String>, Error> {
    if !html.contains("web-link-card-flag=\"true\"") {
        return Ok(Vec::new());
    }

    let mut urls = Vec::new();
    rewrite_str(
        html,
        Settings {
            element_content_handlers: vec![element!("a[web-link-card-flag=\"true\"]", |element| {
                if let Some(href) = element.get_attribute("href") {
                    urls.push(href);
                }
                Ok(())
            })],
            ..Settings::default()
        },
    )
    .map_err(|error| Error::Postprocess {
        slug: slug.to_owned(),
        processor: "web link card URL collection",
        reason: error.to_string(),
    })?;

    Ok(urls)
}

fn transform(
    html: &str,
    cards: &WebLinkCardOgpInput,
    slug: &str,
) -> Result<(String, Vec<ArticleCompileWarning>), Error> {
    if !html.contains("web-link-card-flag=\"true\"") {
        return Ok((html.to_owned(), Vec::new()));
    }

    let mut seen_failed_urls = HashSet::new();
    let mut warnings = Vec::new();
    let transformed_html = rewrite_str(
        html,
        Settings {
            element_content_handlers: vec![element!("a[web-link-card-flag=\"true\"]", |element| {
                let href = element.get_attribute("href").unwrap_or_default();
                if let Some(reason) = cards.failure(&href)
                    && seen_failed_urls.insert(href.clone())
                {
                    warnings.push(ArticleCompileWarning {
                        url: href.clone(),
                        reason: reason.to_owned(),
                    });
                }
                let replacement = cards
                    .get(&href)
                    .map(|metadata| render_card(&href, metadata))
                    .unwrap_or_else(|| render_fallback(&href));
                element.replace(&replacement, ContentType::Html);
                Ok(())
            })],
            ..Settings::default()
        },
    )
    .map_err(|error| Error::Postprocess {
        slug: slug.to_owned(),
        processor: "web link card rendering",
        reason: error.to_string(),
    })?;

    Ok((transformed_html, warnings))
}

fn render_card(href: &str, metadata: &WebLinkCardOgp) -> String {
    let image = metadata
        .image_url
        .as_ref()
        .map(|image| {
            format!(
                "<img class=\"web-link-card__image\" src=\"{}\" alt=\"\" loading=\"lazy\">",
                escape_html(image)
            )
        })
        .unwrap_or_default();
    let description = metadata
        .description
        .as_ref()
        .map(|description| {
            format!(
                "<span class=\"web-link-card__description\">{}</span>",
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        "<a href=\"{}\" class=\"web-link-card\">{image}<span class=\"web-link-card__content\"><span class=\"web-link-card__title\">{}</span>{description}<span class=\"web-link-card__site\">{}</span></span></a>",
        escape_html(href),
        escape_html(&metadata.title),
        escape_html(&metadata.site_name)
    )
}

fn render_fallback(href: &str) -> String {
    format!(
        "<a href=\"{}\" class=\"web-link-card web-link-card--fallback\"><span class=\"web-link-card__fallback\">{}</span></a>",
        escape_html(href),
        escape_html(href)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
