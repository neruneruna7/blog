use std::collections::HashMap;

use super::ogp::WebLinkCardOgp;

/// URLごとのOGP取得結果。
#[derive(Debug, Clone)]
pub struct WebLinkCardOgpInput {
    cards: HashMap<String, Result<WebLinkCardOgp, String>>,
}

impl WebLinkCardOgpInput {
    pub(super) fn new(cards: HashMap<String, Result<WebLinkCardOgp, String>>) -> Self {
        Self { cards }
    }

    /// URLに対応する取得済みOGP情報を返す。
    pub fn get(&self, url: &str) -> Option<&WebLinkCardOgp> {
        self.cards.get(url).and_then(|result| result.as_ref().ok())
    }

    /// URLのOGP取得に失敗した場合、その理由を返す。
    pub fn failure(&self, url: &str) -> Option<&str> {
        self.cards.get(url)?.as_ref().err().map(String::as_str)
    }

    /// URLが一つも指定されなかった場合に`true`を返す。
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}
