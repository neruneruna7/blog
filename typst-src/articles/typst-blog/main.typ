// 記事本文をここに記述する．
#import "../../commons/components/index.typ": *

#set text(lang: "ja")

マークダウンて200種類あんねん．

...というのはともかく．

多くのブログや記事投稿サイトは，マークダウンで記述する．しかし，このブログはTypstで記述する．

なぜか？私のニーズを満たすためであり，究極的に言えば私の好みである．が，それはそれとして色々メリットがある．

これから，実例しつつメリットを見ていこう．

== そもそもTypstとは
LaTeXの代替を目指す組版ソフトウェアである．
#web-link-card("https://typst.app/")

== 早速，真価を発揮！
// フィボナッチ数列でも書くか
#let fib(n) = if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
#let i = 0;
#while i < 20 {
  [#fib(i), ]
  i = i + 1
}
#let sample = [これはフィボナッチ．]
#let j = 0;
#while j < 20 {
  [#sample]
  j = j + 1
}

あなたの見ているサイトは正常です．

これを記事に書くのに，普通どう書くだろうか？フィボナッチ数列を手元で計算してコピペ？　`これはフィボナッチ.`という文字を欲しい回数分コピペ？

いえいえ，こう書くんですよ．
```typ
#let fib(n) = if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
#let i = 0;
#while i < 20 {
  [#fib(i), ]
  i = i + 1
}
#let sample = [これはフィボナッチ．]
#let j = 0;
#while j < 20 {
  [#sample]
  j = j + 1
}
```
はい．そう，typstはプログラミングできる．
あとは読者の方々の想像力に任せよう．

== 真価を発揮2
このブログシステムは，次のような構成で作られている．フロントエンドはAstro @astro + shadcn-ui @shadcn-ui　で実装し，cloudflare @cloudflare でデプロイしている．記事の記述にはtypst @typst を使用し，諸々の処理・変換はRustで実装している @rust.

これを見て，技術記事を書くときに*はぁ〜〜参考文献の参照番号つけるのめんどくっさ*と感じた過去を思い出した人もいるのではないだろうか？

それに対するアンサーはこれだ．*参考文献と参照の手動管理が許されるのはWord @microsoft-word までだよね〜〜!*という煽りである．

実際，こう書いている．
```typ
このブログシステムは，次のような構成で作られている．フロントエンドはAstro @astro + shadcn-ui @shadcn-ui　で実装し，cloudflare @cloudflare でデプロイしている．記事の記述にはtypst @typst を使用し，諸々の処理・変換はRustで実装している @rust.

これを見て，技術記事を書くときに*はぁ〜〜参考文献の参照番号つけるのめんどくっさ*と感じた過去を思い出した人もいるのではないだろうか？

それに対するアンサーはこれだ．*参考文献と参照の手動管理が許されるのはWord @microsoft-word までだよね〜〜!*という煽りである．

...

#bibliography("refs.yml")
```

```yml :refs.yml
typst:
  type: Web
  title: "Typst: The new foundation for documents"
  url:
    value: https://typst.app/
    date: 2026-07-17
astro:
  type: Web
  title: "Astro"
  url:
    value: https://astro.build/
    date: 2026-07-17

cloudflare:
  type: Web
  title: "どこででも接続、保護、構築 | Cloudflare"
  url:
    value: https://www.cloudflare.com/ja-jp/
    date: 2026-07-17

rust:
  type: Web
  title: "Rustプログラミング言語"
  url:
    value: https://rust-lang.org/ja/
    date: 2026-07-17

shadcn-ui:
  type: Web
  title: "The Foundation for your Design System - shadcn/ui"
  url:
    value: https://ui.shadcn.com/
    date: 2026-07-17

microsoft-word:
  type: Web
  title: "Microsoft Word を使用した無料オンライン文書編集 | Microsoft 365"
  url:
    value: https://www.microsoft.com/ja-jp/microsoft-365/word
    date: 2026-07-17
```

参照番号など全く書いていないことにお気づきだろうか？　何を参照するかを`@astro`などで書いているだけである．

== おまけ
#my-grid(columns: 2, gutter: 1em)[
  Typstとは！

  マークダウンではない．

  $"Typst" != "Markdown"$
][
  Typstとは！

  LaTeXと似たような別人である．

  $"Typst" approx "LaTeX"$ (違)
]
技術記事でこんな表記は見たことがないだろう？

数式をKaTeXなどを入れなくても！安定してなんでも表示できる！\
Typstであれば！こんなグリッド表示もできる！　最高！

...いやまぁ実のところ，グリッド表示については誇大広告なのだが．

TypstのHTML export機能は，まだ実験的な機能で，構造化されたHTMLを当てることしかできない．
これまで普通に記事っぽく見えていたのは，自前でCSSを当てたりしているのだ．
`grid`も，中身は自前で用意してhtml/cssでのgridに変換されるようにしている．

= まとめ
- typstでブログ記事を書いている
- 参考文献の管理機能を使えたり，プログラマブルに記事を書けて非常に快適
- ただし，それなりに自前で変換を用意したりしている．
- typstの今後に期待

#bibliography("refs.yml")
