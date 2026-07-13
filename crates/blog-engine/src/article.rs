//! なぜ実装するのか:
//! - 記事メタデータの検証、ドメイン化、公開順序を定義するため。
//!
//! 期待する動作:
//! - TOML入力を検証済み記事へ変換し、日時を検証後の型として保持する。
//! - 記事本体の意味と、一覧表示のための比較規則を分離する。
//!
//! 使い方の概要:
//! - 読み込んだ記事設定を `normalize_articles` または `normalize_article_sources` に渡す。

use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use crate::config::{validate_non_empty, ArticleConfig, ArticleConfigFile};
use crate::error::BlogEngineError;

static SLUG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
        .expect("slug regex must be a valid regular expression")
});

/// なぜ実装するのか:
/// - 検証済み記事の日時を、公開順序の比較に適した値として保持するため。
///
/// 期待する動作:
/// - `published_at` を降順、`slug` を昇順、`id` を昇順で比較する。
/// - 比較結果が等しいとき、3つの比較キーもすべて等しい。
///
/// 使い方の概要:
/// - `Article::publication_order_key` から取得し、記事一覧のsortに渡す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicationOrderKey<'a> {
    published_at: Reverse<&'a DateTime<FixedOffset>>,
    slug: &'a str,
    id: &'a str,
}

/// なぜ実装するのか:
/// - TOML入力を検証済みの値オブジェクトへ変換し、記事の不変条件を型で表すため。
///
/// 期待する動作:
/// - RFC3339として検証された公開日時と更新日時をchronoの型で保持する。
/// - idとslug、タイトル、概要、タグ、公開URLを記事の属性として保持する。
///
/// 使い方の概要:
/// - `normalize_articles` の戻り値として受け取り、出力や本文生成へ渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: DateTime<FixedOffset>,
    pub updated_at: Option<DateTime<FixedOffset>>,
    pub tags: Vec<String>,
    pub url: String,
}

/// なぜ実装するのか:
/// - ソース配置と検証済み記事を同じ処理単位として扱うため。
///
/// 期待する動作:
/// - `article_dir` は設定ファイルの親ディレクトリを示す。
/// - 本文入力パスは記事ディレクトリの `main.typ` であり、出力先は記事slugから決まる。
///
/// 使い方の概要:
/// - `normalize_article_sources` が生成し、本文生成処理へ渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleSource {
    pub(crate) article: Article,
    pub(crate) article_dir: PathBuf,
    pub(crate) main_typ_path: PathBuf,
}

impl Article {
    /// なぜ実装するのか:
    /// - 記事本体へ一覧表示専用の並び順を直接持たせず、比較規則を専用キーへ隔離するため。
    ///
    /// 期待する動作:
    /// - 公開日時降順、slug昇順、id昇順を表す `PublicationOrderKey` を返す。
    ///
    /// 使い方の概要:
    /// - 各記事のキーを `cmp` へ渡し、一覧の並べ替えで使う。
    pub fn publication_order_key(&self) -> PublicationOrderKey<'_> {
        PublicationOrderKey {
            published_at: Reverse(&self.published_at),
            slug: &self.slug,
            id: &self.id,
        }
    }
}

/// なぜ実装するのか:
/// - 複数記事の検証と決定的な公開順序を一箇所に集約するため。
///
/// 期待する動作:
/// - idとslugの重複を拒否し、公開日時降順、同日時ではslug昇順、同slugではid昇順で返す。
///
/// 使い方の概要:
/// - 読み込んだ `ArticleConfig` の一覧を渡し、検証済み `Article` を受け取る。
pub fn normalize_articles(configs: Vec<ArticleConfig>) -> Result<Vec<Article>, BlogEngineError> {
    let mut articles = configs
        .into_iter()
        .map(validate_article)
        .collect::<Result<Vec<_>, BlogEngineError>>()?;

    validate_unique_articles(&articles)?;
    sort_articles(&mut articles);

    Ok(articles)
}

/// なぜ実装するのか:
/// - 記事設定と本文ソースの対応を保ったまま、検証済み記事を決定的な順序で返すため。
///
/// 期待する動作:
/// - 各設定を検証し、idとslugの重複を拒否する。
/// - 公開順序キーで本文ソース全体を並べ替える。
///
/// 使い方の概要:
/// - 記事設定ファイルを読み込んだ後、本文生成処理の前段で呼び出す。
pub(crate) fn normalize_article_sources(
    files: Vec<ArticleConfigFile>,
) -> Result<Vec<ArticleSource>, BlogEngineError> {
    let mut sources = files
        .into_iter()
        .map(|file| {
            let article = validate_article(file.config)?;
            let main_typ_path = file.article_dir.join("main.typ");

            Ok(ArticleSource {
                article,
                article_dir: file.article_dir,
                main_typ_path,
            })
        })
        .collect::<Result<Vec<_>, BlogEngineError>>()?;

    validate_unique_articles(sources.iter().map(|source| &source.article))?;
    sources.sort_by(|left, right| {
        left.article
            .publication_order_key()
            .cmp(&right.article.publication_order_key())
    });

    Ok(sources)
}

/// なぜ実装するのか:
/// - 記事集合におけるidとslugの一意性を検証するため。
///
/// 期待する動作:
/// - 同じidまたはslugが複数の記事に現れた場合、検証エラーを返す。
///
/// 使い方の概要:
/// - 記事単体の値検証が完了した一覧に対して呼び出す。
fn validate_unique_articles<'a>(
    articles: impl IntoIterator<Item = &'a Article>,
) -> Result<(), BlogEngineError> {
    let mut seen_ids = HashSet::new();
    let mut seen_slugs = HashSet::new();

    for article in articles {
        if !seen_ids.insert(article.id.as_str()) {
            return Err(BlogEngineError::validation(format!(
                "article id must be unique: {}",
                article.id
            )));
        }
        if !seen_slugs.insert(article.slug.as_str()) {
            return Err(BlogEngineError::validation(format!(
                "article slug must be unique: {}",
                article.slug
            )));
        }
    }

    Ok(())
}

/// なぜ実装するのか:
/// - 記事一覧の公開順序を、記事本体の意味から分離して適用するため。
///
/// 期待する動作:
/// - `PublicationOrderKey` のOrd実装に従って記事を並べ替える。
///
/// 使い方の概要:
/// - 記事単体の検証と一意性検証が終わった一覧へ適用する。
fn sort_articles(articles: &mut [Article]) {
    articles.sort_by(|left, right| {
        left.publication_order_key()
            .cmp(&right.publication_order_key())
    });
}

/// なぜ実装するのか:
/// - TOML入力を、検証済み日時を持つ記事ドメインモデルへ変換するため。
///
/// 期待する動作:
/// - 必須文字列、slug規則、RFC3339日時を検証する。
/// - 空の更新日時をNoneに変換し、空でない値を型付き日時として保持する。
///
/// 使い方の概要:
/// - `normalize_articles` または `normalize_article_sources` から呼び出す。
fn validate_article(config: ArticleConfig) -> Result<Article, BlogEngineError> {
    validate_non_empty("article.id", &config.id)?;
    validate_non_empty("article.slug", &config.slug)?;
    validate_non_empty("article.title", &config.title)?;
    validate_non_empty("article.description", &config.description)?;
    validate_non_empty("article.published_at", &config.published_at)?;
    validate_slug(&config.slug)?;
    let published_at = parse_rfc3339("article.published_at", &config.published_at)?;

    let updated_at = if config.updated_at.is_empty() {
        None
    } else {
        Some(parse_rfc3339("article.updated_at", &config.updated_at)?)
    };

    let url = format!("/articles/{}/", config.slug);

    Ok(Article {
        id: config.id,
        slug: config.slug,
        title: config.title,
        description: config.description,
        published_at,
        updated_at,
        tags: config.tags,
        url,
    })
}

/// なぜ実装するのか:
/// - slugを記事URLと公開順序の安定した識別子として安全に使うため。
///
/// 期待する動作:
/// - 小文字英数字と単一のハイフン区切りだけを許可する。
///
/// 使い方の概要:
/// - 記事設定をドメインモデルへ変換する前に呼び出す。
pub(crate) fn validate_slug(slug: &str) -> Result<(), BlogEngineError> {
    if !SLUG_REGEX.is_match(slug) {
        return Err(BlogEngineError::validation(format!(
            "article.slug is invalid: {slug}"
        )));
    }
    Ok(())
}

/// なぜ実装するのか:
/// - RFC3339文字列を検証済み日時としてドメインモデルへ渡す境界を一箇所にするため。
///
/// 期待する動作:
/// - 有効な値を `DateTime<FixedOffset>` へ変換し、不正な値を検証エラーとして返す。
///
/// 使い方の概要:
/// - `validate_article` の入力変換時に一度だけ呼び出す。
fn parse_rfc3339(field: &str, value: &str) -> Result<DateTime<FixedOffset>, BlogEngineError> {
    DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .map_err(|source| BlogEngineError::validation(format!("{field} must be RFC3339: {source}")))
}
