//! なぜ実装するのか:
//! - OGP入力を独立したTypstプロセスへ渡し、PNG成果物として確定するため。
//!
//! 期待する動作:
//! - TypstのHTML本文変換とは別のプロセス境界で1200×630 PNGを生成する。
//! - 成功終了だけでなく、PNGシグネチャとIHDR寸法を検証してから成果物を返す。
//!
//! 使い方の概要:
//! - `TypstOgpRenderer::from_project_root` を作り、`OgpRenderer::render` を呼び出す。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::BlogEngineError;
use crate::ogp::{
    generate_ogp_content_hash, OgpArtifact, OgpImageMeta, OgpRenderInput, OgpRenderSettings,
};

/// OGP PNGの契約上の幅である。
pub const OGP_WIDTH_PX: u32 = 1200;

/// OGP PNGの契約上の高さである。
pub const OGP_HEIGHT_PX: u32 = 630;

/// OGP PNGをTypstから出力するときに指定する解像度である。
pub const OGP_PPI: u32 = 72;

/// OGPテンプレートの標準配置である。
pub const OGP_TEMPLATE_PATH: &str = "typst-src/commons/ogp/template.typ";

/// ハッシュに含めるOGPレンダラの実装バージョンである。
pub const OGP_RENDERER_VERSION: &str = "ogp-typst-renderer-v1";

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const IHDR_LENGTH: u32 = 13;
const IHDR_CHUNK_SIZE: usize = 33;

/// なぜ実装するのか:
/// - OGP描画をTypst実行の具体的な方法から分離するため。
///
/// 期待する動作:
/// - 入力値を描画し、検証済みPNGの成果物を返す。
///
/// 使い方の概要:
/// - 本番では `TypstOgpRenderer` を使い、テストでは必要に応じてこの境界をスタブ化する。
pub trait OgpRenderer {
    /// なぜ実装するのか:
    /// - OGP入力からファイルシステム上のPNGを生成する呼び出し境界を固定するため。
    ///
    /// 期待する動作:
    /// - 成功時は出力ファイルの存在、PNG形式、1200×630寸法を満たす成果物を返す。
    /// - Typst開始、コンパイル、出力読み込み、PNG検証の失敗を型付きエラーで返す。
    ///
    /// 使い方の概要:
    /// - `renderer.render(&input, output_path)` のように、検証済み入力と出力先を渡す。
    fn render(
        &self,
        input: &OgpRenderInput,
        output_path: &Path,
    ) -> Result<OgpArtifact, BlogEngineError>;
}

/// なぜ実装するのか:
/// - 現行Typst CLIを用いたOGP PNG生成を本番の実装として提供するため。
///
/// 期待する動作:
/// - `--input key=value` を個別のargvとして構築し、テンプレートへ入力を渡す。
/// - Typstの終了後にPNGヘッダと寸法を検証し、不正な成果物を返さない。
///
/// 使い方の概要:
/// - リポジトリルートから `from_project_root` を呼び出す。
#[derive(Debug, Clone)]
pub struct TypstOgpRenderer {
    template_path: PathBuf,
    project_root: PathBuf,
    typst_binary: PathBuf,
}

impl TypstOgpRenderer {
    /// なぜ実装するのか:
    /// - テンプレートとTypstのプロジェクトルートを明示したレンダラを構築するため。
    ///
    /// 期待する動作:
    /// - 指定値を保持し、実行時までファイル読み込みやプロセス起動を行わない。
    ///
    /// 使い方の概要:
    /// - `TypstOgpRenderer::new(template_path, project_root)` のように呼び出す。
    pub fn new(template_path: impl Into<PathBuf>, project_root: impl Into<PathBuf>) -> Self {
        Self {
            template_path: template_path.into(),
            project_root: project_root.into(),
            typst_binary: PathBuf::from("typst"),
        }
    }

    /// なぜ実装するのか:
    /// - リポジトリ標準のOGPテンプレートを使う本番構成を一つの呼び出しで作るため。
    ///
    /// 期待する動作:
    /// - `<project_root>/typst-src/commons/ogp/template.typ` をテンプレートとして保持する。
    ///
    /// 使い方の概要:
    /// - `TypstOgpRenderer::from_project_root(repo_root)` として使う。
    pub fn from_project_root(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self::new(project_root.join(OGP_TEMPLATE_PATH), project_root)
    }

    /// なぜ実装するのか:
    /// - Typst実行ファイルを差し替え、プロセス開始境界を高速に検証できるようにするため。
    ///
    /// 期待する動作:
    /// - レンダラのテンプレートや入力契約を変えず、実行ファイルだけを差し替える。
    ///
    /// 使い方の概要:
    /// - テストで存在しないパスを渡して開始失敗を検証する。
    pub fn with_typst_binary(mut self, typst_binary: impl Into<PathBuf>) -> Self {
        self.typst_binary = typst_binary.into();
        self
    }

    /// なぜ実装するのか:
    /// - CLIへ渡す引数をシェル文字列ではなくargvの配列として固定するため。
    ///
    /// 期待する動作:
    /// - 改行、カンマ、`=` を含む入力値を一つの引数として保持する。
    /// - タグ配列を件数と添字付きキーに分け、曖昧な区切り文字を使わない。
    ///
    /// 使い方の概要:
    /// - `render` がプロセス起動直前に内部で呼び出す。
    fn command(&self, input: &OgpRenderInput, output_path: &Path) -> Command {
        let mut command = Command::new(&self.typst_binary);
        command
            .arg("compile")
            .arg("--format")
            .arg("png")
            .arg("--root")
            .arg(&self.project_root)
            .arg("--ppi")
            .arg(OGP_PPI.to_string());

        append_input(
            &mut command,
            "ogp_site_title",
            &truncate_display_text(&input.site_title, 48),
        );
        append_input(
            &mut command,
            "ogp_article_title",
            &truncate_display_text(&input.article_title, 84),
        );
        append_input(
            &mut command,
            "ogp_description",
            &truncate_display_text(&input.description, 180),
        );
        let tag_count = input.tags.len().min(6);
        append_input(&mut command, "ogp_tag_count", &tag_count.to_string());
        for (index, tag) in input.tags.iter().take(tag_count).enumerate() {
            append_input(
                &mut command,
                &format!("ogp_tag_{index}"),
                &truncate_display_text(tag, 24),
            );
        }

        command.arg(&self.template_path).arg(output_path);
        command
    }
}

impl OgpRenderer for TypstOgpRenderer {
    fn render(
        &self,
        input: &OgpRenderInput,
        output_path: &Path,
    ) -> Result<OgpArtifact, BlogEngineError> {
        let template = fs::read_to_string(&self.template_path).map_err(|source| {
            BlogEngineError::OgpTemplateRead {
                template_path: self.template_path.clone(),
                output_path: output_path.to_path_buf(),
                source,
            }
        })?;
        let settings = OgpRenderSettings::new(
            OGP_RENDERER_VERSION,
            template,
            OGP_WIDTH_PX,
            OGP_HEIGHT_PX,
            OGP_PPI,
        );
        let content_hash = generate_ogp_content_hash(input, &settings);

        if let Some(parent) = output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|source| BlogEngineError::io(parent.to_path_buf(), source))?;
        }

        let process_output = self
            .command(input, output_path)
            .output()
            .map_err(|source| BlogEngineError::OgpTypstProcessStart {
                template_path: self.template_path.clone(),
                output_path: output_path.to_path_buf(),
                source,
            })?;
        if !process_output.status.success() {
            return Err(BlogEngineError::OgpTypstCompile {
                template_path: self.template_path.clone(),
                output_path: output_path.to_path_buf(),
                status_code: process_output.status.code(),
                stderr: String::from_utf8_lossy(&process_output.stderr)
                    .trim()
                    .to_owned(),
            });
        }

        if !output_path.is_file() {
            return Err(BlogEngineError::OgpOutputMissing {
                template_path: self.template_path.clone(),
                output_path: output_path.to_path_buf(),
                expected_width: OGP_WIDTH_PX,
                expected_height: OGP_HEIGHT_PX,
            });
        }

        let bytes = fs::read(output_path).map_err(|source| BlogEngineError::OgpOutputRead {
            template_path: self.template_path.clone(),
            output_path: output_path.to_path_buf(),
            source,
        })?;
        let dimensions = validate_ogp_png(&bytes, &self.template_path, output_path)?;
        let byte_size = bytes.len() as u64;

        Ok(OgpArtifact {
            file_path: output_path.to_path_buf(),
            image: OgpImageMeta {
                path: public_image_path(output_path),
                width: dimensions.width,
                height: dimensions.height,
                mime_type: "image/png".to_owned(),
                alt: format!("{} | {}", input.article_title, input.site_title),
            },
            content_hash,
            byte_size,
        })
    }
}

/// なぜ実装するのか:
/// - `Command`へ入力キーと値を別々のargvとして追加するため。
///
/// 期待する動作:
/// - Typstが正式に受け付ける `--input key=value` の形を維持する。
/// - 値をシェルへ渡さず、改行や記号をそのまま一つの引数にする。
///
/// 使い方の概要:
/// - `TypstOgpRenderer::command` から各表示値について呼び出す。
fn append_input(command: &mut Command, key: &str, value: &str) {
    command.arg("--input").arg(format!("{key}={value}"));
}

/// なぜ実装するのか:
/// - OGP表示領域へ収めるための派生文字列を、UTF-8の文字境界を保って作るため。
///
/// 期待する動作:
/// - 上限以内の値は変更せず、超過した値だけをUnicodeスカラー値単位で省略する。
/// - 元の `OgpRenderInput` の値は変更しない。
///
/// 使い方の概要:
/// - `command` が表示値をTypstへ渡す直前に呼び出す。
fn truncate_display_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let truncated = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    format!("{truncated}…")
}

/// なぜ実装するのか:
/// - PNG成果物の公開URLを、OS依存のファイルパスから分離して作るため。
///
/// 期待する動作:
/// - 出力ファイル名だけを使い、常に `/` 区切りの公開ルートパスを返す。
///
/// 使い方の概要:
/// - `render` が検証済み出力の `OgpImageMeta.path` を作るときに呼び出す。
fn public_image_path(output_path: &Path) -> String {
    let file_name = output_path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed("ogp.png"));

    format!("/ogp/articles/{file_name}")
}

/// なぜ実装するのか:
/// - Typstが出力したPNGの最小ヘッダを、巨大な画像デコードなしで検証するため。
///
/// 期待する動作:
/// - PNGシグネチャ、IHDRチャンク、幅、高さを安全に読み取る。
/// - 期待寸法と異なる場合は、実寸法を含む型付きエラーを返す。
///
/// 使い方の概要:
/// - Typstプロセス成功後、出力ファイルを読み込んだバイト列へ適用する。
fn validate_ogp_png(
    bytes: &[u8],
    template_path: &Path,
    output_path: &Path,
) -> Result<PngDimensions, BlogEngineError> {
    if bytes.len() < PNG_SIGNATURE.len() || bytes[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(invalid_png_error(
            template_path,
            output_path,
            "PNG signature is missing or invalid",
        ));
    }
    if bytes.len() < IHDR_CHUNK_SIZE {
        return Err(invalid_png_error(
            template_path,
            output_path,
            "PNG does not contain a complete IHDR chunk",
        ));
    }

    let ihdr_length = read_u32(bytes, 8);
    if ihdr_length != IHDR_LENGTH || &bytes[12..16] != b"IHDR" {
        return Err(invalid_png_error(
            template_path,
            output_path,
            "first PNG chunk is not a 13-byte IHDR",
        ));
    }

    let dimensions = PngDimensions {
        width: read_u32(bytes, 16),
        height: read_u32(bytes, 20),
    };
    if dimensions.width != OGP_WIDTH_PX || dimensions.height != OGP_HEIGHT_PX {
        return Err(BlogEngineError::OgpPngDimensions {
            template_path: template_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            expected_width: OGP_WIDTH_PX,
            expected_height: OGP_HEIGHT_PX,
            actual_width: dimensions.width,
            actual_height: dimensions.height,
        });
    }

    Ok(dimensions)
}

/// なぜ実装するのか:
/// - PNGヘッダ不正エラーへ共通の文脈を付与するため。
///
/// 期待する動作:
/// - テンプレート、出力先、期待寸法、理由を失わずに返す。
///
/// 使い方の概要:
/// - `validate_ogp_png` の構造検証失敗から呼び出す。
fn invalid_png_error(template_path: &Path, output_path: &Path, message: &str) -> BlogEngineError {
    BlogEngineError::OgpPngInvalid {
        template_path: template_path.to_path_buf(),
        output_path: output_path.to_path_buf(),
        expected_width: OGP_WIDTH_PX,
        expected_height: OGP_HEIGHT_PX,
        message: message.to_owned(),
    }
}

/// なぜ実装するのか:
/// - 既に長さを検証したPNGヘッダからビッグエンディアン整数を読むため。
///
/// 期待する動作:
/// - IHDRの幅と高さを、バイト順を取り違えずにu32へ変換する。
///
/// 使い方の概要:
/// - `validate_ogp_png` が固定長のIHDRフィールドを読むときに使う。
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// なぜ実装するのか:
/// - PNGのIHDRから読み取った寸法を、検証結果として一時的に保持するため。
///
/// 期待する動作:
/// - PNGヘッダに記録された幅と高さを、検証後の値として保持する。
///
/// 使い方の概要:
/// - `validate_ogp_png` の成功結果として内部的に利用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PngDimensions {
    width: u32,
    height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> OgpRenderInput {
        OgpRenderInput {
            site_title: "site".to_owned(),
            article_title: "title=one\n日本語".to_owned(),
            description: "description,with\nnewlines".to_owned(),
            tags: vec!["one,two".to_owned(), "tag=三".to_owned()],
        }
    }

    fn renderer() -> TypstOgpRenderer {
        TypstOgpRenderer::new("template.typ", "typst-src")
    }

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(&IHDR_LENGTH.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes
    }

    #[test]
    fn command_arguments_keep_special_input_values_as_individual_args() {
        let command = renderer().command(&input(), Path::new("output.png"));
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|pair| {
            pair[0] == "--input" && pair[1] == "ogp_article_title=title=one\n日本語"
        }));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--input" && pair[1] == "ogp_tag_0=one,two" }));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--input" && pair[1] == "ogp_tag_1=tag=三" }));
    }

    #[test]
    fn display_text_is_truncated_without_splitting_utf8_characters() {
        let value = format!("{}終端", "日".repeat(84));

        let truncated = truncate_display_text(&value, 84);

        assert_eq!(truncated.chars().count(), 84);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn failure_to_start_typst_is_reported_as_a_typed_error() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let renderer = TypstOgpRenderer::from_project_root(project_root)
            .with_typst_binary("/path/that/does/not/exist/typst");

        let error = renderer
            .render(&input(), Path::new("target/ogp-process-failure.png"))
            .expect_err("missing Typst binary should fail");

        assert!(matches!(
            error,
            BlogEngineError::OgpTypstProcessStart { .. }
        ));
    }

    #[test]
    fn png_signature_and_ihdr_dimensions_are_validated() {
        let dimensions = validate_ogp_png(
            &png_header(OGP_WIDTH_PX, OGP_HEIGHT_PX),
            Path::new("template.typ"),
            Path::new("output.png"),
        )
        .expect("contract-sized PNG header should be accepted");

        assert_eq!(
            dimensions,
            PngDimensions {
                width: 1200,
                height: 630
            }
        );
    }

    #[test]
    fn invalid_png_signature_is_rejected() {
        let error = validate_ogp_png(
            b"not a png",
            Path::new("template.typ"),
            Path::new("output.png"),
        )
        .expect_err("invalid signature should be rejected");

        assert!(matches!(error, BlogEngineError::OgpPngInvalid { .. }));
    }

    #[test]
    fn png_dimension_mismatch_is_rejected() {
        let error = validate_ogp_png(
            &png_header(1200, 631),
            Path::new("template.typ"),
            Path::new("output.png"),
        )
        .expect_err("wrong PNG dimensions should be rejected");

        assert!(matches!(
            error,
            BlogEngineError::OgpPngDimensions {
                actual_width: 1200,
                actual_height: 631,
                ..
            }
        ));
    }
}
