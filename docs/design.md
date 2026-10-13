[参考:ソフトウェア設計における抽象とメタの違い](https://scrapbox.io/kawasima/%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%E8%A8%AD%E8%A8%88%E3%81%AB%E3%81%8A%E3%81%91%E3%82%8B%E6%8A%BD%E8%B1%A1%E3%81%A8%E3%83%A1%E3%82%BF%E3%81%AE%E9%81%95%E3%81%84)

# 目的
typstでブログを書く．ブログは静的ホストしたい．
そのために？
- typstからHTMLを出力する
- ブログの内容以外を形作る
- その中にtypstから出力した内容を入れる
- ブログの内容(typst)はテキストエディタで編集する．
  - zed
  - vscode
  - など
- 公開された記事のURLにのるものは指定可能
  - hogehoge/articleslug
  - で記事にアクセスできるみたいな

しかし課題がある
- typstのHTMLexportは実験的機能．機能がまだ不十分
  - 自前で実装しなければいけない部分がある
  - CSSなど見た目はこちらで行う必要がある
- OGPをどうするのか？

# ブログ開発環境

- DSL: Typst
- 環境: Zed, VsCode, 任意のテキストエディタ
- 記事自体のOGP: 記事ごとのOGPとメタタグ
- URL: 他記事と重複しなければslugを指定可能
- 公開: 静的ホスト　Cloudflare pages

# 抽象化
- ファイルから読んだ記事情報
- 変換したhtml文字列
- 変換したOGP画像
- 後処理済みhtml
- 処理後のバンドル

# 構造化
- 入力を扱う構造
  - typstコンパイルに必要な情報
    - root (記事ごとに変化なし)
    - typstエントリポイントパス
  - その他設定情報
    - タイトル
    - description
    - slug
    - タグ
    - 公開日
    - 


# 擬似コード

## コンパイル
```txt
let raw_data: Vec<_> = typst-src直下にある記事フォルダを収集．
let validated_data = 異常がないかチェック済みのもの   
  - slugの重複チェックなど．
  - タイトルなどが問題ないか

let html_string = typstコンパイル
let ogp_image = OGPコンパイル

let post_proccessed = 自作の後処理工程．
  - grid対応
  - linkCardでOGP取得

後処理で失敗があれば，警告を標準出力に流す.

let article: Vec<_> = 記事ごとにデータをまとめる
let articles_output = 出力形式に合わせて，変換
- site.jsonの中身
- article-index.jsonの中身

ファイルシステムに書き込み
- コンパイル成果物
  - 記事ごとのhtml, meta.json, assets
  - struct SiteBundle
  - 
```

RawArticle
    ↓ 検証
ValidatedArticle
    ↓ Typstコンパイル
CompiledArticle
    ↓ 後処理
ProcessedArticle
    ↓ サイト全体を集約
SiteBundle
    ↓ シリアライズ・ファイル出力
FileSystem

```rs
struct SiteBundle {
    config: SiteConfig,
    articles: Vec<ArticleBundle>,
    site_index: SiteIndex,
    article_index: ArticleIndex,
}

struct ArticleBundle {
    slug: String,
    html: String,
    metadata: ArticleMetadata,
    assets: Vec<Asset>,
}
```
一旦typstコンパイルまで実装．その後はまた考えよう
