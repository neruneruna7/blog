//! Typstが生成したコードブロックのHTMLを、記事本文向けの構造へ変換する。
//!
//! `pre`要素へCSSの背景画像を敷くだけでは、元の改行と表示上の折り返しを区別できない。
//! そのため、元の改行ごとに`code-line`要素を生成し、罫線の単位をHTMLで明示する。

use lol_html::{
    element, end_tag,
    html_content::{ContentType, TextChunk},
    rewrite_str, text, Settings,
};

use crate::error::BlogEngineError;

const CODE_LINE_OPEN: &str = r#"<span class="code-line">"#;
const CODE_LINE_BREAK: &str = r#"</span><span class="code-line">"#;
const CODE_LINE_CLOSE: &str = "</span>";

/// なぜ実装するのか:
/// - Typstが出力したコードブロックを、記事本文の幅内で表示できる行単位のHTMLへ変換するため。
///
/// 期待する動作:
/// - `pre > code` の元の改行ごとに `code-line` 要素を生成する。
/// - コード中のシンタックスハイライト用タグは保持する。
/// - コードブロック以外のHTMLは変更しない。
///
/// 使い方の概要:
/// - 本文fragmentを保存する前に `process_html` から呼び出す。
pub fn process(html: String) -> Result<String, BlogEngineError> {
    rewrite_str(
        &html,
        Settings {
            element_content_handlers: vec![
                element!("pre > code", |element| {
                    element.prepend(CODE_LINE_OPEN, ContentType::Html);
                    element.on_end_tag(end_tag!(|end| {
                        end.before(CODE_LINE_CLOSE, ContentType::Html);
                        Ok(())
                    }))?;

                    Ok(())
                }),
                text!("pre > code", |text| {
                    replace_source_line_breaks(text);
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
    )
    .map_err(|error| BlogEngineError::html_postprocess(error.to_string()))
}

/// コードテキスト内の元の改行だけを、行要素の境界へ変換する。
///
/// `TextChunk`は完全なテキストノードではないため、チャンク単位で処理する。
/// 既存のテキスト部分はHTMLエンティティを含む可能性があるので、`ContentType::Html`
/// で置換し、元の内容を再エスケープしない。
fn replace_source_line_breaks(text: &mut TextChunk<'_>) {
    let Some(replaced) = text
        .as_str()
        .contains('\n')
        .then(|| text.as_str().replace('\n', CODE_LINE_BREAK))
    else {
        return;
    };

    text.replace(&replaced, ContentType::Html);
}

#[cfg(test)]
mod tests {
    use super::process;

    #[test]
    fn wraps_original_lines_and_preserves_highlight_markup() {
        let html = concat!(
            "<p>before</p>",
            "<pre><code data-lang=\"rs\">",
            "<span style=\"color: #d73948\">fn</span> main() {\n",
            "  <span style=\"color: #d73948\">return</span>;\n",
            "}</code></pre>",
            "<p>after</p>"
        );

        let actual = process(html.to_owned()).unwrap();

        assert_eq!(
            actual,
            concat!(
                "<p>before</p>",
                "<pre><code data-lang=\"rs\">",
                "<span class=\"code-line\"><span style=\"color: #d73948\">fn</span> main() {",
                "</span><span class=\"code-line\">  <span style=\"color: #d73948\">return</span>;",
                "</span><span class=\"code-line\">}</span>",
                "</code></pre>",
                "<p>after</p>"
            )
        );
    }

    #[test]
    fn represents_empty_source_lines() {
        let actual = process("<pre><code>a\n\n b</code></pre>".to_owned()).unwrap();

        assert_eq!(
            actual,
            concat!(
                "<pre><code>",
                "<span class=\"code-line\">a</span>",
                "<span class=\"code-line\"></span>",
                "<span class=\"code-line\"> b</span>",
                "</code></pre>"
            )
        );
    }

    #[test]
    fn leaves_non_code_html_unchanged() {
        let html = "<p>line one\nline two</p>";

        assert_eq!(process(html.to_owned()).unwrap(), html);
    }
}
