// Typst HTML出力でRust後処理が認識するリンクカードを作る．
#let web-link-card(url) = html.elem(
  "a",
  attrs: (
    href: url,
    class: "web-link-card web-link-card--fallback",
    // web-link-card として, 記事エンジンにOGP処理対象と認識させるためのフラグ
    "web-link-card-flag": "true",
  ),
)[
  #html.elem("span", attrs: (class: "web-link-card__fallback"))[#url]
]
