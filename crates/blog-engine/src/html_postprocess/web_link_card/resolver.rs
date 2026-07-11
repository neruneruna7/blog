use std::net::{IpAddr, ToSocketAddrs};
use std::time::Duration;

use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use scraper::{Html, Selector};
use url::Url;

use super::metadata::CardMetadata;

const MAX_REDIRECTS: usize = 5;
const MAX_HTML_BYTES: u64 = 1_048_576;

/// なぜ実装するのか:
/// - メタデータ取得をHTML書換えから分離し、ネットワークを使わないテストを可能にするため。
///
/// 期待する動作:
/// - requested URLのメタデータを正規化して返すか、取得不能理由を返す。
///
/// 使い方の概要:
/// - `WebLinkCardContext` に実装を注入する。
pub trait MetadataResolver: Send + Sync {
    fn resolve(&self, requested_url: &Url) -> Result<CardMetadata, String>;
}

/// なぜ実装するのか:
/// - 制限付きHTTP取得とOGP抽出を本番で提供するため。
///
/// 期待する動作:
/// - publicなHTTP(S)宛先だけを、redirectごとに再検証して取得する。
///
/// 使い方の概要:
/// - `PostprocessContext::production` が生成する。
pub struct HttpMetadataResolver {
    client: Client,
}

impl HttpMetadataResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .redirect(Policy::none())
            .user_agent("blog-engine-web-link-card/0.1")
            .build()
            .expect("reqwest client configuration is valid");
        Self { client }
    }
}

impl Default for HttpMetadataResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataResolver for HttpMetadataResolver {
    fn resolve(&self, requested_url: &Url) -> Result<CardMetadata, String> {
        let mut url = requested_url.clone();
        for redirects in 0..=MAX_REDIRECTS {
            validate_public_url(&url)?;
            let response = self
                .client
                .get(url.clone())
                .send()
                .map_err(|error| error.to_string())?;
            if response.status().is_redirection() {
                if redirects == MAX_REDIRECTS {
                    return Err("redirect limit exceeded".into());
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or("redirect has no Location")?
                    .to_str()
                    .map_err(|_| "invalid redirect Location")?;
                url = url.join(location).map_err(|error| error.to_string())?;
                continue;
            }
            if !response.status().is_success() {
                return Err(format!("HTTP status {}", response.status()));
            }
            if response
                .content_length()
                .is_some_and(|length| length > MAX_HTML_BYTES)
            {
                return Err("HTML response exceeds size limit".into());
            }
            let mut bytes = Vec::new();
            response
                .take(MAX_HTML_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| error.to_string())?;
            if bytes.len() as u64 > MAX_HTML_BYTES {
                return Err("HTML response exceeds size limit".into());
            }
            let text = String::from_utf8(bytes).map_err(|_| "HTML is not UTF-8")?;
            return extract_metadata(requested_url, &url, &text);
        }
        Err("redirect limit exceeded".into())
    }
}

use std::io::Read;

pub fn parse_requested_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only http and https URLs are supported".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URLs with credentials are not supported".into());
    }
    let host = url.host_str().ok_or("URL has no host")?;
    if host.eq_ignore_ascii_case("localhost") {
        return Err("localhost is not allowed".into());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public_ip(ip) {
            return Err("URL has a non-public IP address".into());
        }
    }
    Ok(url)
}

pub fn validate_public_url(url: &Url) -> Result<(), String> {
    parse_requested_url(url.as_str())?;
    let host = url.host_str().ok_or("URL has no host")?;
    let addresses = (host, url.port_or_known_default().ok_or("URL has no port")?)
        .to_socket_addrs()
        .map_err(|error| format!("DNS resolution failed: {error}"))?;
    if addresses
        .into_iter()
        .any(|address| !is_public_ip(address.ip()))
    {
        return Err("URL resolves to a non-public address".into());
    }
    Ok(())
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || ip.is_documentation())
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_unique_local()
                || ip.is_unicast_link_local())
        }
    }
}

fn extract_metadata(requested: &Url, resolved: &Url, html: &str) -> Result<CardMetadata, String> {
    let document = Html::parse_document(html);
    let meta = Selector::parse("meta").expect("valid meta selector");
    let title_selector = Selector::parse("title").expect("valid title selector");
    let mut og_title = None;
    let mut og_description = None;
    let mut og_site = None;
    let mut og_image = None;
    let mut og_image_secure = None;
    let mut description = None;
    for element in document.select(&meta) {
        let value = element.value();
        let content = value
            .attr("content")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        match (value.attr("property"), value.attr("name"), content) {
            (Some("og:title"), _, value) => og_title = value,
            (Some("og:description"), _, value) => og_description = value,
            (Some("og:site_name"), _, value) => og_site = value,
            (Some("og:image"), _, value) => og_image = value,
            (Some("og:image:secure_url"), _, value) => og_image_secure = value,
            (_, Some("description"), value) => description = value,
            _ => {}
        }
    }
    let hostname = resolved
        .host_str()
        .ok_or("resolved URL has no host")?
        .to_owned();
    let title = og_title
        .or_else(|| {
            document
                .select(&title_selector)
                .next()
                .map(|node| node.text().collect::<String>().trim().to_owned())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| hostname.clone());
    let image_url = og_image_secure
        .or(og_image)
        .and_then(|value| resolved.join(&value).ok())
        .map(|url| url.to_string());
    Ok(CardMetadata {
        schema_version: 1,
        requested_url: requested.to_string(),
        resolved_url: resolved.to_string(),
        title,
        description: og_description.or(description),
        site_name: og_site.unwrap_or(hostname),
        image_url,
        fetched_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_image_is_resolved() {
        let metadata = extract_metadata(
            &Url::parse("https://example.com/a").unwrap(),
            &Url::parse("https://example.com/path/").unwrap(),
            "<meta property=\"og:image\" content=\"image.png\">",
        )
        .unwrap();
        assert_eq!(
            metadata.image_url.as_deref(),
            Some("https://example.com/path/image.png")
        );
    }
    #[test]
    fn title_falls_back_to_html_title_then_hostname() {
        let requested = Url::parse("https://example.com/a").unwrap();
        assert_eq!(
            extract_metadata(&requested, &requested, "<title>Document title</title>")
                .unwrap()
                .title,
            "Document title"
        );
        assert_eq!(
            extract_metadata(&requested, &requested, "").unwrap().title,
            "example.com"
        );
    }
    #[test]
    fn unsafe_urls_are_rejected() {
        assert!(parse_requested_url("file:///etc/passwd").is_err());
        assert!(parse_requested_url("http://localhost/").is_err());
        assert!(parse_requested_url("http://127.0.0.1/").is_err());
    }
}
