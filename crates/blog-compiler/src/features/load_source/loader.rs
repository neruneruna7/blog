//! 実験
//! 動詞＋接尾辞erのついたものは，そのコロケーション内でのエントリポイントを示すように．
//! 期待する効果
//! - フォルダを見た際に，どこから辿ればいいかわかりやすくなる
//!- ここがエントリポイントだということをわかりやすくする．
use super::{
    article::RawArticle,
    error::Error,
    site::{RawSiteBundle, RawSiteConfig},
};
use std::path::Path;

pub struct SourceLoader;

impl SourceLoader {
    pub fn load(
        root: impl AsRef<Path>,
        article_entry_point: impl AsRef<Path>,
    ) -> Result<RawSiteBundle, Error> {
        let root = root.as_ref();
        let articles = RawArticle::read_all(root, article_entry_point)?;
        let site_config = RawSiteConfig::read(root)?;

        Ok(RawSiteBundle {
            articles,
            site_config,
        })
    }
}
