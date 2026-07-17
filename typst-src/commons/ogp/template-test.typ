// なぜ実装するのか:
// - 記事メタデータから、共有用の固定サイズOGP画像を生成するため。
//
// 期待する動作:
// - サイト名、記事タイトル、概要、タグを一枚のPNGへ描画する。
// - 表示領域を固定し、長い値は縮小と表示用省略でページ外への流出を防ぐ。
//
// 使い方の概要:
// - Rustの`TypstOgpRenderer`が`--input`で値を渡し、Typst CLIでPNGへコンパイルする。

#set page(width: 1200pt, height: 630pt, margin: 0pt)

#let font-logo = (
  "Kaisei Decol",
  // "Shippori Mincho",
  // "Noto Serif JP",
  // "Libertinus Serif",
)
#let font-heading = (
  "Shippori Mincho",
  // "Noto Serif JP",
  // "Libertinus Serif",
)
#let font-body = (
  "Noto Serif JP",
  // "Shippori Mincho",
  // "Libertinus Serif",
)
#let font-decor = (
  "Hina Mincho",
  // "Shippori Mincho",
  // "Noto Serif JP",
  // "Libertinus Serif",
)
#let font-sans = (
  "Geist",
  "Noto Sans JP",
)

#set text(
  font: font-body,
  fill: rgb("#f8fafc"),
)

#let read-input(name) = sys.inputs.at(name, default: "")

#let site-title = "彗星記憶連晶"
#let article-title = "テスト記事タイトル．コミュニケーション"
#let description = "これはテストの説明である"
#let tag-count-input = int(3)
#let tag-count = if tag-count-input > 6 { 6 } else { tag-count-input }
// #let tags = range(tag-count).map(i => read-input("ogp_tag_" + str(i)))
#let tag-line = ("1", "2")

#let title-size = if article-title.len() <= 28 {
  72pt
} else if article-title.len() <= 48 {
  60pt
} else {
  48pt
}

#let description-size = if description.len() <= 80 {
  26pt
} else if description.len() <= 130 {
  22pt
} else {
  18pt
}

#rect(
  width: 100%,
  height: 100%,
  fill: rgb("#0f172a"),
  inset: 64pt,
)[
  #block(width: 100%, height: 50pt)[
    #text(
      font: font-logo,
      size: 28pt,
      weight: 500,
      fill: rgb("#93c5fd"),
    )[#site-title]
  ]

  #v(24pt)

  #block(width: 100%, height: 215pt)[
    #text(font: font-heading, size: title-size, weight: 600)[#article-title]
  ]

  #v(16pt)

  #block(width: 100%, height: 110pt)[
    #text(
      font: font-body,
      size: description-size,
      fill: rgb("#cbd5e1"),
    )[#description]
  ]

  #v(16pt)

  #block(width: 100%, height: 32pt, )[
    #text(font: font-sans, size: 20pt, fill: rgb("#93c5fd"))[#tag-line]
  ]
]
