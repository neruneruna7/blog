リンクカード取得時の外部OGP画像をキャッシュしておく

### 目的
不要な外部アクセスを行わない．
これによって，コンパイルを高速化．
記事執筆中の反復速度を上げ，より快適に書けるようにする．


### 要件
- web-link-cardで外部から取得するOGP画像をキャッシュして再利用する
- これは開発記事ビルド時にキャッシュが利用される
- 本番記事ビルドでは利用しない

### 実装
- キャッシュデータは，blogキャッシュで取得すればいいか？
  - urlまたはそのハッシュ化文字列をキーにして，キャッシュ判定を行う
  - generated/.chache の中に，json形式でキャッシュ状況を記録する
  - generated/.chache/web-link-card の中に，キャッシュされたファイルを保持する
- feature, web-link-card-ogp-inputを作り，そこで外部のOGPを取得する．
  - キャッシュ状況データを受け取る．
  - キャッシュ済みのものはアクセスしない．
- article_transformでは，その対応関係マップを

```
    let html_articles = parsed_source
        .articles
        .iter()
        .filter(|(_, article)| !release || PublicationFilter::is_publication_target(article))
        .filter(|(slug, _)| build_plan.needs_html(slug))
        .map(|(slug, article)| (slug.clone(), article.clone()))
        .collect::<HashMap<_, _>>();
    // ここにキャッシュ判定を入れるしかないか？
    let compiled_html = ArticleTransformer::transform_all(html_articles)?;
```
