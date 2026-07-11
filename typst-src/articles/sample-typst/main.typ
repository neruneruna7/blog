
// #import "@preview/numbly:0.1.0": numbly
// // #set heading(numbering: numbly("{1}.", default: "1.1"))
// #set heading(numbering: none)

// // --- Font & Language Settings ---
// #show heading: set text(lang: "ja", font: "Harano Aji Gothic")
// #show outline: set text(lang: "ja", font: "Harano Aji Gothic")
// #set text(lang: "ja", font: "Harano Aji Gothic")
// // #show math.equation: set text(lang: "ja",font: "Harano Aji Gothic")
// #show math.equation: set text(lang: "en", font: "New Computer Modern Math")

// // 数式中の日本語テキストを日本語フォントにするためのヘルパー関数
// #let jatext(it) = text(font: "Harano Aji Gothic", it)

// #set par(
//   first-line-indent: (
//     amount: 1em,
//     all: true,
//   ),
// )
// #set page(
//   paper: "a4",
//   margin: 25mm,
// )

#import "../../commons/components/index.typ": *

#set text(lang: "ja")

#let link-card(
  url,
  title: none,
  description: none,
  image: none,
) = context {
  if target() == "html" {
    block(
      html.article(
        class: "article-link-card",
      )[
        #html.a(
          href: url,
          rel: "noopener",
          class: "article-link-card__link",
        )[
          #html.div(class: "article-link-card__body")[
            #if title != none {
              html.h3(class: "article-link-card__title")[#title]
            }

            #if description != none {
              html.p(class: "article-link-card__description")[
                #description
              ]
            }

            #html.span(class: "article-link-card__url")[
              #url
            ]
          ]

          #if image != none {
            html.img(
              src: image,
              alt: "",
              class: "article-link-card__image",
            )
          }
        ]
      ],
    )
  } else {
    block[
      #if title != none {
        strong(title)
      } else {
        url
      }

      #linebreak()
      #link(url)[#url]
    ]
  }
}

= はじめに
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
  stroke: 1pt,
  columns: 3,

  [a], [b], [c],
)

#figure(caption: "表1")[
  #table(
    columns: 3,
    [a], [b], [c],
  )
]

#rect(stroke: 2pt, fill: rgb(100, 100, 100, 57))[
  これは長方形
]

#image("assets/17.png")

=== 見出し3
あああああ
感じ漢字凹凸

$
  alpha + beta = X
$
#link-card("github.com/neruneruna7/blog/tree/feat/astro-react-rust-typst")


#blog-rect()

aaa

#web-link-card("https://zenn.dev/avaintelligence/articles/b7d4743a448485#%E3%81%AF%E3%81%98%E3%82%81%E3%81%AB")

参照 @AGIC 参照 @cpa 参照 @guo2026gradient-inversion


#bibliography("refs.yml", full: true)
