//! なぜ実装するのか:
//! - Typstが付与したリンクカード用anchorを、ビルド時のメタデータで置換するため。
//!
//! 期待する動作:
//! - 外部失敗ではクリック可能なfallbackを残し、ビルドを継続する。
//!
//! 使い方の概要:
//! - `process` に本文fragmentと `WebLinkCardContext` を渡す。

mod cache;
mod metadata;
mod processor;
mod resolver;

pub use metadata::CardMetadata;
pub use processor::{process, WebLinkCardContext};
pub use resolver::{HttpMetadataResolver, MetadataResolver};
