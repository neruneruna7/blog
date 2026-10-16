//! typstエントリポイントのファイル名とか
//! typstコンパイルルートとか
//! アプリ全体に関わる定数を置く
//! それぞれの関心ごとのところに定数を置こうとしていたが，
//! 結局全体に関わる設定なのだから共有の場所に配置する．

pub const TYP_COMPILE_ROOT: &str = "typst-src";
pub const ARTICLE_ENTRY_POINT: &str = "main.typ";
pub const OGP_MAIN_FILE: &str = "commons/ogp/template.typ";
pub const ARTIFACT_ROOT: &str = "generated";
