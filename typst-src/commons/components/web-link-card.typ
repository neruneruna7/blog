// Typst HTML出力ではRust後処理用のリンクカード候補を生成し，
// PDF・SVG・PNG出力では通常のリンクとして表示する．
#let web-link-card(url) = context {
  if target() == "html" {
    html.elem(
      "a",
      attrs: (
        href: url,
        class: "web-link-card web-link-card--fallback",
        // web-link-card として, 記事エンジンにOGP処理対象と認識させるためのフラグ
        "web-link-card-flag": "true",
      ),
    )[
      #html.elem(
        "span",
        attrs: (class: "web-link-card__fallback"),
      )[#url]
    ]
  } else {
    link(url)[#url]
  }
}
