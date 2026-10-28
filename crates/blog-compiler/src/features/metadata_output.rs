//! 解析済みソースから公開用 JSON を生成して書き込む。

pub mod error;
mod writer;

pub use writer::MetadataWriter;
