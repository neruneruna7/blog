//! なぜ実装するのか:
//! - Typstが生成した本文fragmentへ、明示的で順序固定の変換を適用するため。
//!
//! 期待する動作:
//! - 各processorはHTML文字列を受け取り、新しいHTML文字列を返す。
//!
//! 使い方の概要:
//! - `process_html` を本文抽出後、fragment検証前に呼び出す。

pub mod context;
pub mod web_link_card;

pub use context::PostprocessContext;

use crate::error::BlogEngineError;

/// なぜ実装するのか:
/// - 後処理の実行順序をコード上で固定し、暗黙の登録機構を避けるため。
///
/// 期待する動作:
/// - 現在はリンクカードだけを処理し、将来のprocessorもここへ明示的に追加する。
///
/// 使い方の概要:
/// - Typst出力の本文fragmentを渡す。
pub fn process_html(html: String, context: &PostprocessContext) -> Result<String, BlogEngineError> {
    let html = web_link_card::process(html, &context.web_link_card)?;
    Ok(html)
}
