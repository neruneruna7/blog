//! なぜ実装するのか:
//! - リンクカード後処理に必要な依存だけを、明示的に渡すため。
//!
//! 期待する動作:
//! - 汎用サービスロケータを使わず、cacheとresolverの境界を固定する。
//!
//! 使い方の概要:
//! - 本番では `PostprocessContext::production` を使い、テストではresolverを差し替える。

use std::path::PathBuf;

use super::web_link_card::{HttpMetadataResolver, WebLinkCardContext};

/// なぜ実装するのか:
/// - HTML後処理の依存を記事生成の外部境界から注入するため。
///
/// 期待する動作:
/// - processorごとの文脈を名前付きフィールドとして保持する。
///
/// 使い方の概要:
/// - 通常は `production` を使う。
pub struct PostprocessContext {
    pub web_link_card: WebLinkCardContext,
}

impl PostprocessContext {
    /// なぜ実装するのか:
    /// - キャッシュ位置とHTTP resolverを本番用に一箇所で決めるため。
    ///
    /// 期待する動作:
    /// - 作業ディレクトリ直下の `.cache/blog-engine/web-link-cards` を利用する。
    ///
    /// 使い方の概要:
    /// - 通常の `build` 内部から呼び出す。
    pub fn production() -> Self {
        Self {
            web_link_card: WebLinkCardContext::new(
                PathBuf::from(".cache/blog-engine/web-link-cards"),
                Box::new(HttpMetadataResolver::new()),
            ),
        }
    }
}
