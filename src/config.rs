use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// サイト設定を表すドメインモデルである。
///
/// # なぜ実装するのか
/// SiteConfig.tomlをRust側で読み，URL，feed，sitemap，HTMLメタ情報の正本として扱うためである。
///
/// # 期待する動作
/// 必須フィールドを保持し，TOMLに欠落や型不一致があればdeserializeで失敗する。
///
/// # 使い方の概要
/// `manifest::load_manifest`から読み込み，renderとbuild処理へ渡す。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub author: String,
    pub language: String,
    pub timezone: String,
}

/// 記事設定を表すドメインモデルである。
///
/// # なぜ実装するのか
/// ArticleConfig.tomlをRust側で検証し，URL生成，index生成，feed生成の正本として扱うためである。
///
/// # 期待する動作
/// 必須フィールドを保持し，idとslugの一意性検証に利用できる。
///
/// # 使い方の概要
/// TOMLからdeserializeし，ArticleManifestへ正規化してビルド処理へ渡す。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArticleConfig {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub published_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub draft: bool,
}
