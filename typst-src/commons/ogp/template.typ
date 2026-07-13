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
#set text(
  font: ("Hiragino Sans", "Cica"),
  fill: rgb("#f8fafc"),
)

#let read-input(name) = sys.inputs.at(name, default: "")

#let site-title = read-input("ogp_site_title")
#let article-title = read-input("ogp_article_title")
#let description = read-input("ogp_description")
#let tag-count-input = int(read-input("ogp_tag_count"))
#let tag-count = if tag-count-input > 6 { 6 } else { tag-count-input }
#let tags = range(tag-count).map(i => read-input("ogp_tag_" + str(i)))
#let tag-line = tags.map(tag => "#" + tag).join("  ")

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
  #block(width: 100%, height: 42pt, clip: true)[
    #text(size: 28pt, weight: "bold", fill: rgb("#93c5fd"))[#site-title]
  ]

  #v(24pt)

  #block(width: 100%, height: 215pt, clip: true)[
    #text(size: title-size, weight: "bold")[#article-title]
  ]

  #v(16pt)

  #block(width: 100%, height: 110pt, clip: true)[
    #text(size: description-size, fill: rgb("#cbd5e1"))[#description]
  ]

  #v(16pt)

  #block(width: 100%, height: 32pt, clip: true)[
    #text(size: 20pt, fill: rgb("#93c5fd"))[#tag-line]
  ]
]
