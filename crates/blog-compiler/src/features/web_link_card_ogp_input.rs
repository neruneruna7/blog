//! 外部リンクカードからOGP情報を取得し、指定された場合にキャッシュする。

mod cache_record;
mod input;
mod loader;
mod ogp;

pub use input::WebLinkCardOgpInput;
pub use loader::WebLinkCardOgpLoader;
pub use ogp::WebLinkCardOgp;
