//! リンクカードを作るために，OGP表示できるように解決．
//!

use std::{
    collections::HashMap,
    io::Read,
    net::{IpAddr, ToSocketAddrs},
    time::Duration,
};

use lol_html::{Settings, element, html_content::ContentType, rewrite_str, text};
use regex::Regex;
use reqwest::{Url, blocking::Client, redirect::Policy};

use crate::features::article_transform::{error::Error, output::ArticleCompileWarning};

const MAX_REDIRECTS: usize = 5;
const MAX_HTML_BYTES: u64 = 1_048_576;

struct Resolver {
    client: Client,
}

impl Resolver {
    fn production() -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .redirect(Policy::none())
            .user_agent("blog-compiler-web-link-card/0.1")
            .build()
            .expect("web link card HTTP client configuration is valid");

        Self { client }
    }

    fn resolve(&self, urls: &[String]) -> ResolvedCards {
        let mut cards = HashMap::with_capacity(urls.len());
        for href in urls {
            cards
                .entry(href.clone())
                .or_insert_with(|| resolve_card(href, self));
        }

        ResolvedCards(cards)
    }
}

struct ResolvedCards(HashMap<String, Result<CardMetadata, String>>);

impl ResolvedCards {
    fn failures(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().filter_map(|(href, result)| {
            result
                .as_ref()
                .err()
                .map(|reason| (href.as_str(), reason.as_str()))
        })
    }
}

pub(super) struct Processor {
    resolver: Resolver,
}

impl Processor {
    pub(super) fn production() -> Self {
        Self {
            resolver: Resolver::production(),
        }
    }

    pub(super) fn process(&self, html: &str, slug: &str) -> Result<Output, Error> {
        let urls = collect_urls(html, slug)?;
        let cards = self.resolver.resolve(&urls);
        let html = transform(html, &cards, slug)?;
        let warnings = cards
            .failures()
            .map(|(url, reason)| ArticleCompileWarning {
                url: url.to_owned(),
                reason: reason.to_owned(),
            })
            .collect::<Vec<_>>();

        Ok(Output { html, warnings })
    }
}

pub(super) struct Output {
    pub(super) html: String,
    pub(super) warnings: Vec<ArticleCompileWarning>,
}

fn collect_urls(html: &str, slug: &str) -> Result<Vec<String>, Error> {
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

fn transform(html: &str, cards: &ResolvedCards, slug: &str) -> Result<String, Error> {
    if !html.contains("web-link-card-flag=\"true\"") {
        return Ok(html.to_owned());
    }

    rewrite_str(
        html,
        Settings {
            element_content_handlers: vec![element!("a[web-link-card-flag=\"true\"]", |element| {
                let href = element.get_attribute("href").unwrap_or_default();
                let replacement = cards
                    .0
                    .get(&href)
                    .and_then(|result| result.as_ref().ok())
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
    })
}

fn resolve_card(href: &str, resolver: &Resolver) -> Result<CardMetadata, String> {
    let requested_url = parse_requested_url(href)?;
    fetch_metadata(&resolver.client, &requested_url)
}

fn parse_requested_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only http and https URLs are supported".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URLs with credentials are not supported".to_owned());
    }

    let host = url.host_str().ok_or("URL has no host")?;
    if host.eq_ignore_ascii_case("localhost") {
        return Err("localhost is not allowed".to_owned());
    }
    if host
        .parse::<IpAddr>()
        .is_ok_and(|address| !is_public_ip(address))
    {
        return Err("URL has a non-public IP address".to_owned());
    }

    Ok(url)
}

fn validate_public_url(url: &Url) -> Result<(), String> {
    parse_requested_url(url.as_str())?;
    let host = url.host_str().ok_or("URL has no host")?;
    let port = url.port_or_known_default().ok_or("URL has no port")?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("DNS resolution failed: {error}"))?;
    if addresses
        .into_iter()
        .any(|address| !is_public_ip(address.ip()))
    {
        return Err("URL resolves to a non-public address".to_owned());
    }

    Ok(())
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            !(address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_unspecified()
                || address.is_multicast()
                || address.is_broadcast()
                || address.is_documentation())
        }
        IpAddr::V6(address) => {
            !(address.is_loopback()
                || address.is_unspecified()
                || address.is_multicast()
                || address.is_unique_local()
                || address.is_unicast_link_local())
        }
    }
}

fn fetch_metadata(client: &Client, requested_url: &Url) -> Result<CardMetadata, String> {
    let mut resolved_url = requested_url.clone();
    for redirect_count in 0..=MAX_REDIRECTS {
        validate_public_url(&resolved_url)?;
        let response = client
            .get(resolved_url.clone())
            .send()
            .map_err(|error| error.to_string())?;

        if response.status().is_redirection() {
            if redirect_count == MAX_REDIRECTS {
                return Err("redirect limit exceeded".to_owned());
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .ok_or("redirect has no Location")?
                .to_str()
                .map_err(|_| "invalid redirect Location")?;
            resolved_url = resolved_url
                .join(location)
                .map_err(|error| error.to_string())?;
            continue;
        }

        if !response.status().is_success() {
            return Err(format!("HTTP status {}", response.status()));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_HTML_BYTES)
        {
            return Err("HTML response exceeds size limit".to_owned());
        }

        let mut bytes = Vec::new();
        response
            .take(MAX_HTML_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_HTML_BYTES {
            return Err("HTML response exceeds size limit".to_owned());
        }
        let html = String::from_utf8(bytes).map_err(|_| "HTML is not UTF-8")?;
        return extract_metadata(&resolved_url, &html);
    }

    Err("redirect limit exceeded".to_owned())
}

fn extract_metadata(resolved_url: &Url, html: &str) -> Result<CardMetadata, String> {
    let entity_regex = Regex::new(r"&(?:#[xX][0-9a-fA-F]+|#\d+|[a-zA-Z][a-zA-Z0-9]+);")
        .expect("HTML entity regex is valid");
    let mut open_graph_title = None;
    let mut open_graph_description = None;
    let mut open_graph_site_name = None;
    let mut open_graph_image = None;
    let mut open_graph_secure_image = None;
    let mut description = None;
    let mut document_title = String::new();
    rewrite_str(
        html,
        Settings {
            element_content_handlers: vec![
                element!("meta", |element| {
                    let property = element.get_attribute("property");
                    let name = element.get_attribute("name");
                    let content = element
                        .get_attribute("content")
                        .map(|value| decode_entities(value.trim(), &entity_regex))
                        .filter(|value| !value.is_empty());
                    match (property.as_deref(), name.as_deref(), content) {
                        (Some("og:title"), _, value) => open_graph_title = value,
                        (Some("og:description"), _, value) => open_graph_description = value,
                        (Some("og:site_name"), _, value) => open_graph_site_name = value,
                        (Some("og:image"), _, value) => open_graph_image = value,
                        (Some("og:image:secure_url"), _, value) => open_graph_secure_image = value,
                        (_, Some("description"), value) => description = value,
                        _ => {}
                    }
                    Ok(())
                }),
                text!("title", |text| {
                    document_title.push_str(text.as_str());
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
    )
    .map_err(|error| error.to_string())?;

    let hostname = resolved_url
        .host_str()
        .ok_or("resolved URL has no host")?
        .to_owned();
    let title = open_graph_title
        .or_else(|| (!document_title.trim().is_empty()).then_some(document_title))
        .map(|title| decode_entities(title.trim(), &entity_regex))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| hostname.clone());
    let image_url = open_graph_secure_image
        .or(open_graph_image)
        .and_then(|image| resolved_url.join(&image).ok())
        .map(|url| url.to_string());

    Ok(CardMetadata {
        title,
        description: open_graph_description.or(description),
        site_name: open_graph_site_name.unwrap_or(hostname),
        image_url,
    })
}

fn decode_entities(value: &str, entity_regex: &Regex) -> String {
    entity_regex
        .replace_all(value, |captures: &regex::Captures<'_>| {
            let entity = &captures[0];
            decode_entity(entity).unwrap_or_else(|| entity.to_owned())
        })
        .into_owned()
}

fn decode_entity(entity: &str) -> Option<String> {
    let name = entity.strip_prefix('&')?.strip_suffix(';')?;
    let decoded = match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        _ if name.starts_with("#x") || name.starts_with("#X") => {
            char::from_u32(u32::from_str_radix(&name[2..], 16).ok()?)?
        }
        _ if name.starts_with('#') => char::from_u32(name[1..].parse().ok()?)?,
        _ => return None,
    };
    Some(decoded.to_string())
}

fn render_card(href: &str, metadata: &CardMetadata) -> String {
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

#[derive(Debug, Clone)]
struct CardMetadata {
    title: String,
    description: Option<String>,
    site_name: String,
    image_url: Option<String>,
}
