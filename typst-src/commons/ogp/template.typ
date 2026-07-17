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

#let color-background = rgb(240, 240, 240)
#let color-foreground = rgb("#0c212c")
#let color-muted-foreground = rgb("#546671")

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
  "DejaVu Sans",
)

#set text(
  font: font-body,
  fill: color-foreground,
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
  fill: color-background,
  inset: 64pt,
)[

  #v(24pt)
  // 説明
  #block(width: 100%, height: 100pt, stroke: 0pt)[
    #align(center+bottom)[
    #text(
      font: font-body,
      size: description-size,
      fill: color-muted-foreground,
    )[#description]
  ]
]

  // タイトル
  #block(width: 100%, height: 215pt,stroke: 0pt, inset: 8pt)[
    #align(center+horizon)[
      #text(font: font-heading, size: title-size, weight: 600)[#article-title]
    ]

    #let lineLen = 40%
    #grid(columns: (1fr, 1fr, 1fr), align: center, gutter: 12pt)[
      #align(right, 
      line(length: lineLen)
      )
    ][
      #block(width: 90%, height: 32pt, )[
        #text(font: font-sans, size: 20pt, fill: color-muted-foreground)[#tag-line ]
      ]
    ][
      #align(left, 
      line(length: lineLen)
      )
    ]
  ]

  #v(32pt)
    #block(width: 100%, height: 50pt)[
    #align(center+horizon)[
      #text(
        font: font-logo,
        size: 38pt,
        weight: 500,
        fill: color-foreground,
      )[#site-title]
    ]
  ]
]
