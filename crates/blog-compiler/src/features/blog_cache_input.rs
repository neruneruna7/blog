//! 現在の入力指紋、既存成果物、前回成功した指紋を取得する。

pub mod error;
mod hash;
mod loader;

pub use loader::{BlogCacheInput, BlogCacheLoader};
