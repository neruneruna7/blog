use serde::{Deserialize, Serialize};

/// なぜ実装するのか:
/// - OGP由来の値を描画・キャッシュ共通の正規化済み型で扱うため。
///
/// 期待する動作:
/// - requested URLと最終URLを区別し、任意メタデータを明示的に表す。
///
/// 使い方の概要:
/// - resolverが返し、processorがカードHTMLを生成する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMetadata {
    pub schema_version: u8,
    pub requested_url: String,
    pub resolved_url: String,
    pub title: String,
    pub description: Option<String>,
    pub site_name: String,
    pub image_url: Option<String>,
    pub fetched_at: String,
}
