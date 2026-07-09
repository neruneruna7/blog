use std::fs;
use std::path::Path;

use chrono::{SecondsFormat, Utc};
use uuid::Uuid;

use crate::error::{BlogError, PathContext, Result};
use crate::paths::ProjectPaths;

/// 新規記事の雛形を作成する。
///
/// # なぜ実装するのか
/// id，slug，必要ファイルを一貫した形式で生成し，手作業による欠落を避けるためである。
///
/// # 期待する動作
/// 正規化タイトルのディレクトリを作り，ArticleConfig.toml，main.typ，ogp.typ，assets/を生成する。
///
/// # 使い方の概要
/// CLIの`new "Article title"`から呼び出す。
pub fn create_article(root: impl AsRef<Path>, title: &str) -> Result<()> {
    let paths = ProjectPaths::new(root);
    let slug = normalize_title(title);
    let article_dir = paths.articles_dir().join(&slug);
    if article_dir.exists() {
        return Err(BlogError::ArticleAlreadyExists(article_dir));
    }

    fs::create_dir_all(article_dir.join("assets")).at(article_dir.join("assets"))?;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let config = format!(
        "id = \"{}\"\nslug = \"{}\"\ntitle = \"{}\"\ndescription = \"\"\ntags = []\npublished_at = \"{}\"\nupdated_at = \"{}\"\ndraft = true\n",
        Uuid::now_v7(),
        slug,
        escape_toml(title),
        now,
        now
    );
    fs::write(article_dir.join("ArticleConfig.toml"), config)
        .at(article_dir.join("ArticleConfig.toml"))?;
    fs::write(article_dir.join("main.typ"), format!("= {title}\n"))
        .at(article_dir.join("main.typ"))?;
    fs::write(
        article_dir.join("ogp.typ"),
        format!("#set page(width: 1200pt, height: 630pt)\n= {title}\n"),
    )
    .at(article_dir.join("ogp.typ"))?;
    Ok(())
}

/// タイトルをslug/ディレクトリ名へ正規化する。
///
/// # なぜ実装するのか
/// `new`コマンドが外部仕様として安定した初期slugを生成するためである。
///
/// # 期待する動作
/// ASCII英数字を小文字化し，区切りを単一ハイフンへ変換する。
///
/// # 使い方の概要
/// 記事作成前にタイトル文字列へ適用する。
pub fn normalize_title(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }

    let slug = out.trim_matches('-').to_owned();
    if slug.is_empty() {
        "untitled".to_owned()
    } else {
        slug
    }
}

fn escape_toml(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
