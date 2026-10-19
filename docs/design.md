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

# 命名
ドメイン用語と対応する命名の正本は [`domain-language.toml`](domain-language.toml) とする．


# 擬似コード

## コンパイル
```txt
let source = SourceLoader::load()
let parsed_source = SourceParser::parse(source)

let cache_input = BlogCacheLoader::load(parsed_source)
let build_plan = BuildCache::new(&cache_input)
let mut cache_writer = BuildCacheWriter::new(cache_input, generated)

let articles_to_transform = parsed_source.articles
  .filter(|article| build_plan.needs_html(article.slug))
let html = ArticleTransformer::transform_all(articles_to_transform)
for (slug, html) in html {
  ArticleWriter::write(slug, html, generated)
  cache_writer.record_html_success(slug)
}

for article in parsed_source.articles
  .filter(|article| build_plan.needs_ogp(article.slug)) {
  let png = OgpGenerator::generate(article, parsed_source.site)
  OgpWriter::write(article.slug, png, generated)
  cache_writer.record_ogp_success(article.slug)
}

MetadataWriter::write(parsed_source, generated)

ArticleWriter::remove_obsolete(parsed_source.slugs, generated)
MetadataWriter::remove_obsolete(parsed_source, generated)
OgpWriter::remove_obsolete(parsed_source.slugs, generated)
cache_writer.remove_obsolete(parsed_source.slugs)
```

BlogCacheLoaderは現在の入力指紋、既存成果物の検証結果、前回成功した指紋を返す。BuildCacheは受け取った値だけから再生成対象を決める。Manifestの読み込みと形式の検証はLoaderに隠蔽される。

HTML、OGP、メタデータは独立した出力パイプラインである。ArticleWriterは`articles/<slug>/body.html`、OgpWriterは`public-root/ogp/articles/<slug>.png`、MetadataWriterは`site.json`・`article-index.json`・記事ごとの`meta.json`をそれぞれ生成先へ書く。MetadataWriterの入力はParsedSourceだけであり、OGP画像の公開パスはSlugから決定する。

各成果物Writerはファイル単位で一時ファイルを置き換える。HTMLまたはOGPの指紋は、対応するWriterの成功後にBuildCacheWriterが単一の`manifest.json`へ個別に保存する。逐次実行のためManifest更新にプロセス内ロックは設けない。

記事削除時はArticleWriterが`body.html`、MetadataWriterが`meta.json`、OgpWriterがOGP画像を削除し、BuildCacheWriterが不要なManifest項目を削除する。

## 新規記事作成
```
let selected_template = CLIによって選択されたテンプレート. load_template

let articles = 現在ある記事一覧を取得する．slugの重複防止のため．

let new_slug = 他と重複しないslug．

let new_article_config = (new_slug)新規記事の設定．CLIで指定がなければデフォルト値を入れる．デフォルト値はDefaultトレイトで指定する．

let new_article = データ組み立て

new_article.create()
```


## 異常系
読み込み時にエラーがあったらどうするか？
その時点で処理を落とすか？
それとも正常なものは最後まで実行し，すべての警告を集計した上で表示するか？

一旦，異常時点で落とせばいいや．
