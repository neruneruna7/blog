//! 実験
//! 動詞＋接尾辞erのついたものは，そのコロケーション内でのエントリポイントを示すように．
//! 期待する効果
//! - フォルダを見た際に，どこから辿ればいいかわかりやすくなる
//!- ここがエントリポイントだということをわかりやすくする．
use super::{
    TYP_COMPILE_ROOT,
    article::RawArticle,
    error::Error,
    site::{RawSiteBundle, RawSiteConfig},
};

pub struct Loader;

impl Loader {
    pub fn load() -> Result<RawSiteBundle, Error> {
        let root = TYP_COMPILE_ROOT;
        let articles = RawArticle::read_all(root)?;
        let site_config = RawSiteConfig::read(root)?;

        Ok(RawSiteBundle {
            articles,
            site_config,
        })
    }
}
