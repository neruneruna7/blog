use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

pub fn article_compile() -> Result<(), ()> {
    let article_inputs = todo!("入力ファイル群");
    let validated_article_inputs = todo!("問題ないことが確認された入力データ");
    let compiled = todo!("コンパイル");

    todo!("ファイル書き込み")
}

// 複数ユースケースを横断する必要があるだろうが...
// まぁ，その時にどこに置くべきか見えてくるだろう．
const ROOT: &str = "typst-src";

#[derive(Debug, Clone)]
struct RowArticleInput {
    entry_point: PathBuf,
    config: RowConfig,
}

impl RowArticleInput {
    fn read_articles(root: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let root = root.as_ref();
        let entry_point = root.join("index.typ");
        let config_path = root.join("config.toml");
        let config_contents = fs::read_to_string(config_path)?;
        let config = toml::from_str(&config_contents)?;

        Ok(Self {
            entry_point,
            config,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RowConfig {
    slug: String,
    title: String,
    description: String,
    tags: Vec<String>,
    published_at: String,
}

// バリデーション後

#[derive(Debug, Clone)]
struct ArticleInput {}

// #[derive(Debug, Clone)]
// struct Config {
//     slug: Slug,
//     title: Title,
//     description: Description,
//     tags: Vec<Tag>,
//     published_at: DateTime<FixedOffset>,
// }
// #[derive(Debug, Clone)]
// struct Slug(String);

// impl Slug {
//     fn new(_0: String, existing_slugs: &HashSet<String>) -> Result<Self, ()> {
//         let validated = existing_slugs.contains(&_0);
//         if validated { Ok(Self(_0)) } else { Err(()) }
//     }
// }

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::RowArticleInput;

    #[test]
    /// ファイルが読めることを確認
    fn reads_article_entry_point_and_config() {
        let root = tempdir().expect("temporary directory should be created");
        fs::write(
            root.path().join("config.toml"),
            r#"
slug = "rust-article"
title = "Rust Article"
description = "An article about Rust."
tags = ["rust", "compiler"]
published_at = "2026-10-11T12:00:00+09:00"
"#,
        )
        .expect("config should be written");

        let article =
            RowArticleInput::read_articles(root.path()).expect("article input should be read");

        assert_eq!(article.entry_point, root.path().join("index.typ"));
        assert_eq!(article.config.slug, "rust-article");
        assert_eq!(article.config.title, "Rust Article");
        assert_eq!(article.config.description, "An article about Rust.");
        assert_eq!(
            article.config.tags,
            vec![String::from("rust"), String::from("compiler")]
        );
        assert_eq!(article.config.published_at, "2026-10-11T12:00:00+09:00");
    }
}
