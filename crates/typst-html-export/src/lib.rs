use std::{
    collections::HashSet,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use typst::{
    Feature, Library, LibraryExt, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Dict, Duration, Str, Value},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::Font,
    utils::{LazyHash, Scalar},
};
use typst_html::HtmlDocument;
use typst_kit::fonts::FontStore;
use typst_layout::PagedDocument;
use typst_render::{RenderOptions, render};

const OGP_WIDTH_PX: u32 = 1200;
const OGP_HEIGHT_PX: u32 = 630;
const OGP_TAG_LIMIT: usize = 6;

/// OGPテンプレートへ渡す表示データである。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgpInput {
    pub site_title: String,
    pub article_title: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Typstプロジェクト内の文書をHTMLへ変換する。
///
/// `main_file` が相対パスの場合は `project_root` から解決する。
/// Typstの相対importも同じプロジェクトルートを基準に解決される。
/// ```
/// let html_string = typst_html_export::compile_html(
///    "typst-src",
///    "articles/typst-blog/main.typ",
/// )?;
/// ```
pub fn compile_html(
    project_root: impl AsRef<Path>,
    main_file: impl AsRef<Path>,
) -> Result<String, TypstHtmlExportError> {
    let world = FsWorld::new(project_root.as_ref(), main_file.as_ref())?;
    let document = typst::compile::<HtmlDocument>(&world)
        .output
        .map_err(|diagnostics| TypstHtmlExportError::Compile(format!("{diagnostics:?}")))?;

    typst_html::html(&document, &Default::default())
        .map_err(|diagnostics| TypstHtmlExportError::Html(format!("{diagnostics:?}")))
}

/// Typst OGP文書をPNGバイト列へ変換する。
///
/// ファイル保存を呼び出し側へ委ねるため、PNGデータをメモリ上の値として返す。
pub fn compile_ogp_png(
    project_root: impl AsRef<Path>,
    main_file: impl AsRef<Path>,
    input: &OgpInput,
) -> Result<Vec<u8>, TypstHtmlExportError> {
    let library = Library::builder().with_inputs(typst_inputs(input)).build();
    let world = FsWorld::with_library(project_root.as_ref(), main_file.as_ref(), library)?;
    let document = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|diagnostics| TypstHtmlExportError::Compile(format!("{diagnostics:?}")))?;

    if document.pages().len() != 1 {
        return Err(TypstHtmlExportError::OgpPageCount(document.pages().len()));
    }

    let pixmap = render(
        &document.pages()[0],
        &RenderOptions {
            pixel_per_pt: Scalar::new(1.0),
            render_bleed: false,
        },
    );
    if pixmap.width() != OGP_WIDTH_PX || pixmap.height() != OGP_HEIGHT_PX {
        return Err(TypstHtmlExportError::OgpDimensions {
            actual_width: pixmap.width(),
            actual_height: pixmap.height(),
        });
    }

    pixmap
        .encode_png()
        .map_err(|error| TypstHtmlExportError::PngEncode(error.to_string()))
}

fn typst_inputs(input: &OgpInput) -> Dict {
    let mut values = Dict::new();
    insert_text_input(
        &mut values,
        "ogp_site_title",
        &display_input(&input.site_title, 48),
    );
    insert_text_input(
        &mut values,
        "ogp_article_title",
        &display_input(&input.article_title, 84),
    );
    insert_text_input(
        &mut values,
        "ogp_description",
        &display_input(&input.description, 180),
    );

    let tags = normalize_tags(&input.tags);
    insert_text_input(&mut values, "ogp_tag_count", &tags.len().to_string());
    for (index, tag) in tags.into_iter().enumerate() {
        insert_text_input(
            &mut values,
            &format!("ogp_tag_{index}"),
            &truncate_display_text(&tag, 24),
        );
    }

    values
}

fn display_input(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_display_text(&normalized, max_chars)
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    tags.iter()
        .map(|tag| tag.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|tag| !tag.is_empty() && seen.insert(tag.clone()))
        .take(OGP_TAG_LIMIT)
        .collect()
}

fn insert_text_input(values: &mut Dict, key: &str, value: &str) {
    values.insert(Str::from(key), Value::Str(Str::from(value)));
}

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

#[derive(Debug)]
pub enum TypstHtmlExportError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    ProjectRootNotDirectory(PathBuf),
    MainFileNotFile(PathBuf),
    MainFileOutsideProject {
        project_root: PathBuf,
        main_file: PathBuf,
    },
    InvalidMainFilePath {
        path: PathBuf,
        reason: String,
    },
    Compile(String),
    Html(String),
    OgpPageCount(usize),
    OgpDimensions {
        actual_width: u32,
        actual_height: u32,
    },
    PngEncode(String),
}

impl fmt::Display for TypstHtmlExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::ProjectRootNotDirectory(path) => {
                write!(
                    formatter,
                    "project root is not a directory: {}",
                    path.display()
                )
            }
            Self::MainFileNotFile(path) => {
                write!(
                    formatter,
                    "main Typst file is not a file: {}",
                    path.display()
                )
            }
            Self::MainFileOutsideProject {
                project_root,
                main_file,
            } => write!(
                formatter,
                "main Typst file {} is outside project root {}",
                main_file.display(),
                project_root.display()
            ),
            Self::InvalidMainFilePath { path, reason } => write!(
                formatter,
                "invalid main Typst file path {}: {reason}",
                path.display()
            ),
            Self::Compile(diagnostics) => {
                write!(formatter, "Typst compilation failed: {diagnostics}")
            }
            Self::Html(diagnostics) => write!(formatter, "Typst HTML export failed: {diagnostics}"),
            Self::OgpPageCount(actual) => {
                write!(
                    formatter,
                    "OGP document must have exactly one page (got {actual})"
                )
            }
            Self::OgpDimensions {
                actual_width,
                actual_height,
            } => write!(
                formatter,
                "OGP image must be {OGP_WIDTH_PX}x{OGP_HEIGHT_PX} pixels (got {actual_width}x{actual_height})"
            ),
            Self::PngEncode(message) => write!(formatter, "failed to encode OGP PNG: {message}"),
        }
    }
}

impl Error for TypstHtmlExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

struct FsWorld {
    root: PathBuf,
    main: FileId,
    library: LazyHash<Library>,
    fonts: FontStore,
}

impl FsWorld {
    fn new(project_root: &Path, main_file: &Path) -> Result<Self, TypstHtmlExportError> {
        let library = Library::builder()
            .with_features([Feature::Html].into_iter().collect())
            .build();
        Self::with_library(project_root, main_file, library)
    }

    fn with_library(
        project_root: &Path,
        main_file: &Path,
        library: Library,
    ) -> Result<Self, TypstHtmlExportError> {
        let root = canonicalize(project_root)?;
        if !root.is_dir() {
            return Err(TypstHtmlExportError::ProjectRootNotDirectory(root));
        }

        let requested_main = if main_file.is_absolute() {
            main_file.to_path_buf()
        } else {
            root.join(main_file)
        };
        let main_path = canonicalize(&requested_main)?;
        if !main_path.starts_with(&root) {
            return Err(TypstHtmlExportError::MainFileOutsideProject {
                project_root: root,
                main_file: main_path,
            });
        }
        if !main_path.is_file() {
            return Err(TypstHtmlExportError::MainFileNotFile(main_path));
        }

        let main_path_virtual = VirtualPath::virtualize(&root, &main_path).map_err(|error| {
            TypstHtmlExportError::InvalidMainFilePath {
                path: main_path.clone(),
                reason: format!("{error:?}"),
            }
        })?;
        let main = FileId::new(RootedPath::new(VirtualRoot::Project, main_path_virtual));
        let mut fonts = FontStore::new();
        fonts.extend(typst_kit::fonts::system());
        fonts.extend(typst_kit::fonts::embedded());

        Ok(Self {
            root,
            main,
            library: LazyHash::new(library),
            fonts,
        })
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if !matches!(id.root(), VirtualRoot::Project) {
            return Err(FileError::NotFound(id.vpath().get_without_slash().into()));
        }

        let path = id
            .vpath()
            .realize(&self.root)
            .map_err(|_| FileError::NotFound(id.vpath().get_without_slash().into()))?;
        let canonical_path = path
            .canonicalize()
            .map_err(|error| FileError::from_io(error, &path))?;
        if !canonical_path.starts_with(&self.root) {
            return Err(FileError::NotFound(id.vpath().get_without_slash().into()));
        }

        Ok(canonical_path)
    }
}

impl World for FsWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<typst::text::FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let path = self.resolve(id)?;
        let text = fs::read_to_string(&path).map_err(|error| FileError::from_io(error, &path))?;

        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        let data = fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;

        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn canonicalize(path: &Path) -> Result<PathBuf, TypstHtmlExportError> {
    path.canonicalize()
        .map_err(|source| TypstHtmlExportError::Io {
            path: path.to_path_buf(),
            source,
        })
}
