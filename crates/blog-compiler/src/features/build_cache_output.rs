//! 生成済み成果物の指紋をManifestへ記録する。

mod error;
mod writer;

pub use error::Error;
pub use writer::BuildCacheWriter;
