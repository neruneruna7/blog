//! html変換をした後，ファイルシステムに書き込むためのデータを表す．
pub mod article;
pub mod error;
pub mod site;
pub mod writer;

pub use writer::ArtifactWriter;
