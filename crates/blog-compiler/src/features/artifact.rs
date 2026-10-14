//! html変換をした後，ファイルシステムに書き込むためのデータを表す．
pub mod article;
// artifactの下にartifactがあると，clippyに警告を出されるので回避策として
pub mod artifact_model;
pub mod assembler;
pub mod error;
pub mod site;

pub use artifact_model::Artifact;
pub use assembler::ArtifactAssembler;
