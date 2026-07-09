
#import "@preview/numbly:0.1.0": numbly
// #set heading(numbering: numbly("{1}.", default: "1.1"))
#set heading(numbering: none)

// --- Font & Language Settings ---
#show heading: set text(lang: "ja", font: "Harano Aji Gothic")
#show outline: set text(lang: "ja", font: "Harano Aji Gothic")
#set text(lang: "ja", font: "Harano Aji Gothic")
// #show math.equation: set text(lang: "ja",font: "Harano Aji Gothic")
#show math.equation: set text(lang: "en", font: "New Computer Modern Math")

// 数式中の日本語テキストを日本語フォントにするためのヘルパー関数
#let jatext(it) = text(font: "Harano Aji Gothic", it)

#set par(
  first-line-indent: (
    amount: 1em,
    all: true,
  ),
)
#set page(
  paper: "a4",
  margin: 25mm,
)

#align(center)[
  #text(size: 14pt, weight: "bold")[2026年度 レポート]
  \
  #text(size: 14pt)[
    学籍番号: 11111 氏名: aaaaa
  ]
]

= 課題

#lorem(300)


== 見出し2

#table(
  columns: 3,
  [a], [b], [c]
)

#figure(caption: "表1")[
  #table(
    columns: 3,
    [a], [b], [c]
  )
]

#rect(stroke: 2pt, fill: rgb(100, 100, 100, 57))[
  これは長方形
]

#image("assets/17.png")

=== 見出し3
あああああ
感じ漢字凹凸
