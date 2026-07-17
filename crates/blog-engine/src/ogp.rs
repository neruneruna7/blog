//! なぜ実装するのか:
//! - OGP描画に必要な表示値と、生成成果物の公開契約をTypst実行から分離するため。
//!
//! 期待する動作:
//! - 検証済みサイト・記事モデルから描画専用の入力値を構築する。
//! - 描画入力とテンプレート・設定値から決定的なSHA-256と公開パスを生成する。
//! - ファイルシステム上の成果物情報と公開URLの相対パスを混同しない。
//!
//! 使い方の概要:
//! - `OgpRenderInput::from_models` で入力を作り、
//!   `generate_ogp_content_hash` と `build_ogp_public_path` を後続の描画処理から呼び出す。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::article::Article;
use crate::output::SiteOutput;

/// OGPハッシュの入力スキーマを識別するバージョンである。
pub const OGP_HASH_SCHEMA_VERSION: &str = "ogp-render-contract-v1";

/// なぜ実装するのか:
/// - OGPへ表示する値だけを、検証済みのサイト・記事モデルから取り出すため。
///
/// 期待する動作:
/// - 表示文字列の前後と連続する空白を正規化する。
/// - タグの空要素と重複を除き、最初の出現順を保持する。
/// - 元の `SiteOutput` と `Article` を変更しない。
///
/// 使い方の概要:
/// - `OgpRenderInput::from_models(&site, &article)` で構築し、ハッシュや後続の描画境界へ渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgpRenderInput {
    pub site_title: String,
    pub article_title: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl OgpRenderInput {
    /// なぜ実装するのか:
    /// - サイトと記事のドメインモデルから、OGP描画に必要な最小の値だけを受け渡すため。
    ///
    /// 期待する動作:
    /// - `SiteOutput.title`、`Article.title`、`Article.description`、`Article.tags` を採用する。
    /// - OGP表示用の派生値だけを正規化し、入力モデルの所有値は変更しない。
    ///
    /// 使い方の概要:
    /// - TOML検証後の `SiteOutput` と `Article` を借用して呼び出す。
    pub fn from_models(site: &SiteOutput, article: &Article) -> Self {
        Self {
            site_title: normalize_display_text(&site.title),
            article_title: normalize_display_text(&article.title),
            description: normalize_display_text(&article.description),
            tags: normalize_tags(&article.tags),
        }
    }
}

/// なぜ実装するのか:
/// - OGP描画結果を変化させるテンプレートとレンダラ設定を、ハッシュ入力へ明示的に含めるため。
///
/// 期待する動作:
/// - バージョン、テンプレート内容、幅、高さ、PPIを保持する。
/// - 同じ値を保持する設定は同じハッシュ入力を形成する。
///
/// 使い方の概要:
/// - OGP描画処理が使用するテンプレート内容と設定値を `new` へ渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgpRenderSettings {
    pub renderer_version: String,
    pub template: String,
    pub width: u32,
    pub height: u32,
    pub ppi: u32,
}

impl OgpRenderSettings {
    /// なぜ実装するのか:
    /// - OGPハッシュの対象となる描画設定を一つの値オブジェクトとして構築するため。
    ///
    /// 期待する動作:
    /// - 引数の値をそのまま保持し、テンプレートや描画設定の変更を失わない。
    ///
    /// 使い方の概要:
    /// - `OgpRenderSettings::new("ogp-renderer-v1", template, 1200, 630, 144)` のように呼び出す。
    pub fn new(
        renderer_version: impl Into<String>,
        template: impl Into<String>,
        width: u32,
        height: u32,
        ppi: u32,
    ) -> Self {
        Self {
            renderer_version: renderer_version.into(),
            template: template.into(),
            width,
            height,
            ppi,
        }
    }
}

/// なぜ実装するのか:
/// - 最終JSONへ載せるOGP画像の公開メタデータを、ファイルシステム情報から分離するため。
///
/// 期待する動作:
/// - `path` は公開ルート基準のURLパスであり、絶対URLやOS依存のファイルパスではない。
/// - JSONへシリアライズするとき、フィールド名をcamelCaseへ変換する。
///
/// 使い方の概要:
/// - OGP成果物を確定した後、公開パス・画像サイズ・MIME型・代替テキストを設定する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgpImageMeta {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub alt: String,
}

/// なぜ実装するのか:
/// - OGP画像の実ファイルと、公開時に参照する画像メタデータを同じ生成単位で保持するため。
///
/// 期待する動作:
/// - `file_path` はファイルシステム上の配置先を表す。
/// - `image.path` は公開ルート基準のURLパスを表し、絶対URLを保持しない。
/// - `content_hash` は成果物名の元となる完全なSHA-256文字列を表す。
///
/// 使い方の概要:
/// - 後続の画像生成処理で実ファイルを書き出した後、画像メタデータとバイト数を詰めて返す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgpArtifact {
    pub file_path: PathBuf,
    pub image: OgpImageMeta,
    pub content_hash: String,
    pub byte_size: u64,
}

/// なぜ実装するのか:
/// - OGP表示値と描画情報の変更を、キャッシュ可能な成果物識別子へ変換するため。
///
/// 期待する動作:
/// - 固定フィールド順のJSONへシリアライズしてからSHA-256を計算する。
/// - サイト名、記事タイトル、概要、タグ列、テンプレート、幅、高さ、PPI、バージョンを含める。
/// - `HashMap`の反復順やOSのパス区切りに依存しない。
///
/// 使い方の概要:
/// - `OgpRenderInput` と `OgpRenderSettings` を渡し、戻り値を `OgpArtifact.content_hash` と公開パス生成へ使う。
pub fn generate_ogp_content_hash(input: &OgpRenderInput, settings: &OgpRenderSettings) -> String {
    let hash_input = OgpHashInput {
        schema_version: OGP_HASH_SCHEMA_VERSION,
        renderer_version: &settings.renderer_version,
        site_title: &input.site_title,
        article_title: &input.article_title,
        description: &input.description,
        tags: &input.tags,
        template: &settings.template,
        width: settings.width,
        height: settings.height,
        ppi: settings.ppi,
    };
    let serialized = serde_json::to_vec(&hash_input)
        .expect("OGP hash input contains only serializable primitive values");
    let digest = Sha256::digest(serialized);

    format!("{digest:x}")
}

/// なぜ実装するのか:
/// - 完全なコンテンツハッシュから、キャッシュ可能で短い公開画像パスを決めるため。
///
/// 期待する動作:
/// - `/ogp/articles/<slug>-<hash-prefix>.png` の形式で常に `/` を使う。
/// - `hash` の先頭12文字だけをファイル名へ含め、絶対URLは生成しない。
///
/// 使い方の概要:
/// - 検証済み記事slugと `generate_ogp_content_hash` の戻り値を渡す。
pub fn build_ogp_public_path(slug: &str, content_hash: &str) -> String {
    let hash_prefix = content_hash.chars().take(12).collect::<String>();

    format!("/ogp/articles/{slug}-{hash_prefix}.png")
}

/// なぜ実装するのか:
/// - 表示用文字列の正規化をOGP入力の派生処理へ閉じ込めるため。
///
/// 期待する動作:
/// - Unicodeの空白を区切りとして扱い、前後空白と連続空白を一つの半角空白へ変換する。
/// - 元の文字列を変更せず、正規化後の所有文字列を返す。
///
/// 使い方の概要:
/// - `OgpRenderInput::from_models` の表示文字列変換から呼び出す。
fn normalize_display_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// なぜ実装するのか:
/// - タグ列を表示用の値へ変換し、ハッシュ入力を不要な空要素や重複から安定させるため。
///
/// 期待する動作:
/// - 各タグを表示用に正規化し、空要素を除く。
/// - 重複を最初の出現位置で一度だけ残し、入力順を維持する。
///
/// 使い方の概要:
/// - `OgpRenderInput::from_models` から記事タグを借用して呼び出す。
fn normalize_tags(tags: &[String]) -> Vec<String> {
    tags.iter().fold(Vec::new(), |mut normalized, tag| {
        let tag = normalize_display_text(tag);
        if !tag.is_empty() && !normalized.iter().any(|existing| existing == &tag) {
            normalized.push(tag);
        }
        normalized
    })
}

/// なぜ実装するのか:
/// - ハッシュ入力のフィールド境界と順序を、構造体の定義として固定するため。
///
/// 期待する動作:
/// - JSONの配列でタグ順を保持し、文字列の連結による曖昧性を生じさせない。
/// - `HashMap`など順序が不定なコンテナを使用しない。
///
/// 使い方の概要:
/// - `generate_ogp_content_hash` 内部で一時的に構築し、serde_jsonへ渡す。
#[derive(Serialize)]
struct OgpHashInput<'a> {
    schema_version: &'static str,
    renderer_version: &'a str,
    site_title: &'a str,
    article_title: &'a str,
    description: &'a str,
    tags: &'a [String],
    template: &'a str,
    width: u32,
    height: u32,
    ppi: u32,
}
