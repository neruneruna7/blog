use std::{
    collections::{HashMap, HashSet},
    io::Read,
    net::{IpAddr, ToSocketAddrs},
    path::{Path, PathBuf},
    time::Duration,
};

use lol_html::{Settings, element, rewrite_str, text};
use regex::Regex;
use reqwest::{Url, blocking::Client, redirect::Policy};
use sha2::{Digest, Sha256};

use crate::commons::atomic_file::write_atomic;

use super::{
    cache_record::{CACHE_SCHEMA_VERSION, CacheRecord},
    input::WebLinkCardOgpInput,
    ogp::WebLinkCardOgp,
};

const MAX_REDIRECTS: usize = 5;
const MAX_HTML_BYTES: u64 = 1_048_576;
const CACHE_DIRECTORY: &str = ".cache";
const WEB_LINK_CARD_DIRECTORY: &str = "web-link-card";

/// URLごとのOGP情報をキャッシュから読み込み、未登録分だけ外部から取得する。
pub struct WebLinkCardOgpLoader;

impl WebLinkCardOgpLoader {
    /// URL群のOGP情報を解決する。
    ///
    /// `cache_root`を指定すると、成功した情報を`<cache_root>/.cache/web-link-card`へ保存して再利用する。
    /// キャッシュに有効期限はなく、削除または形式変更まで再利用する。`None`の場合はキャッシュを読み書きしない。
    /// 外部取得に失敗したURLは保存せず、失敗理由を返す入力に含める。
    ///
    /// # Examples
    ///
    /// ```
    /// use blog_compiler::features::web_link_card_ogp_input::WebLinkCardOgpLoader;
    ///
    /// let input = WebLinkCardOgpLoader::load(&[], None);
    /// assert!(input.is_empty());
    /// ```
    pub fn load(urls: &[String], cache_root: Option<&Path>) -> WebLinkCardOgpInput {
        let cache_directory =
            cache_root.map(|root| root.join(CACHE_DIRECTORY).join(WEB_LINK_CARD_DIRECTORY));
        let mut results = HashMap::with_capacity(urls.len());
        let mut pending_urls = Vec::with_capacity(urls.len());
        let mut seen_urls = HashSet::with_capacity(urls.len());

        for url in urls {
            if !seen_urls.insert(url.as_str()) {
                continue;
            }

            let cached = cache_directory
                .as_deref()
                .and_then(|directory| read_cached(directory, url));
            if let Some(metadata) = cached {
                results.insert(url.clone(), Ok(metadata));
            } else {
                pending_urls.push(url);
            }
        }

        if !pending_urls.is_empty() {
            let client = Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .redirect(Policy::none())
                .user_agent("blog-compiler-web-link-card/0.1")
                .build();

            for url in pending_urls {
                let result = client
                    .as_ref()
                    .map_err(ToString::to_string)
                    .and_then(|client| resolve_card(url, client));
                if let (Some(directory), Ok(metadata)) = (cache_directory.as_deref(), &result) {
                    write_cached(directory, url, metadata);
                }
                results.insert(url.clone(), result);
            }
        }

        WebLinkCardOgpInput::new(results)
    }
}

fn read_cached(directory: &Path, url: &str) -> Option<WebLinkCardOgp> {
    let path = cache_path(directory, url);
    let contents = match std::fs::read(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "リンクカードのOGPキャッシュを読み取れないため再取得");
            return None;
        }
    };

    let record = match serde_json::from_slice::<CacheRecord>(&contents) {
        Ok(record) => record,
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "リンクカードのOGPキャッシュが不正なため再取得");
            return None;
        }
    };
    if record.schema_version != CACHE_SCHEMA_VERSION || record.url != url {
        tracing::warn!(path = %path.display(), "リンクカードのOGPキャッシュが古いかURLと一致しないため再取得");
        return None;
    }

    Some(record.metadata)
}

fn write_cached(directory: &Path, url: &str, metadata: &WebLinkCardOgp) {
    let path = cache_path(directory, url);
    let record = CacheRecord {
        schema_version: CACHE_SCHEMA_VERSION,
        url: url.to_owned(),
        metadata: metadata.clone(),
    };
    let contents = match serde_json::to_vec(&record) {
        Ok(contents) => contents,
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "リンクカードのOGPキャッシュを直列化できない");
            return;
        }
    };
    if let Err(error) = write_atomic(&path, &contents) {
        tracing::warn!(path = %path.display(), %error, "リンクカードのOGPキャッシュを書き込めない");
    }
}

fn cache_path(directory: &Path, url: &str) -> PathBuf {
    let hash = format!("{:x}", Sha256::digest(url.as_bytes()));
    directory.join(format!("{hash}.json"))
}

fn resolve_card(href: &str, client: &Client) -> Result<WebLinkCardOgp, String> {
    let requested_url = parse_requested_url(href)?;
    fetch_metadata(client, &requested_url)
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

fn fetch_metadata(client: &Client, requested_url: &Url) -> Result<WebLinkCardOgp, String> {
    let mut resolved_url = requested_url.clone();
    for redirect_count in 0..=MAX_REDIRECTS {
        validate_public_url(&resolved_url)?;
        let response = client.get(resolved_url.clone()).send().map_err(|error| {
            tracing::warn!(
                method = "GET",
                url = %resolved_url,
                redirect_count,
                error = %error,
                "OGPメタデータ取得のHTTPアクセスに失敗"
            );
            error.to_string()
        })?;
        tracing::info!(
            method = "GET",
            url = %resolved_url,
            redirect_count,
            status = %response.status(),
            "OGPメタデータ取得のHTTP応答"
        );

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

fn extract_metadata(resolved_url: &Url, html: &str) -> Result<WebLinkCardOgp, String> {
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

    Ok(WebLinkCardOgp {
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
