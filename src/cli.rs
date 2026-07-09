use clap::{Parser, Subcommand};

/// blog-protoのCLI定義である。
///
/// # なぜ実装するのか
/// `build`，`check`，`new`を外部から安定したコマンドとして実行するためである。
///
/// # 期待する動作
/// サブコマンドと引数を検証し，対応する処理へ渡せる値を生成する。
///
/// # 使い方の概要
/// `Cli::parse()`でコマンドライン引数から生成し，`Commands`で分岐する。
#[derive(Debug, Parser)]
#[command(name = "blog-proto")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// blog-protoが提供するサブコマンドである。
///
/// # なぜ実装するのか
/// ビルド，検証，新規記事作成の責務を明示的に分けるためである。
///
/// # 期待する動作
/// 各variantがCLIの外部仕様と一対一に対応する。
///
/// # 使い方の概要
/// `match`で処理を選択し，必要な引数を各モジュールへ渡す。
#[derive(Debug, Subcommand)]
pub enum Commands {
    Build,
    Check,
    New { title: String },
}
