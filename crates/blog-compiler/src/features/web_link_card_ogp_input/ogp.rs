use serde::{Deserialize, Serialize};

/// リンクカードを表示するために取得したOGP情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebLinkCardOgp {
    pub title: String,
    pub description: Option<String>,
    pub site_name: String,
    pub image_url: Option<String>,
}
