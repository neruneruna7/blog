use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use typst::{
    Feature, Library, LibraryExt, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::Font,
    utils::LazyHash,
};
use typst_html::HtmlDocument;
use typst_kit::fonts::FontStore;

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
        let library = Library::builder()
            .with_features([Feature::Html].into_iter().collect())
            .build();

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
