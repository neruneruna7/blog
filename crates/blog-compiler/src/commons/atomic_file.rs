use std::{fs, io, path::Path};

use uuid::Uuid;

/// 同じディレクトリの一時ファイルへ書き込み、完成後に出力先を置き換える。
///
/// 読み手が書き込み途中の生成ファイルを観測しないようにする。
///
/// # Errors
///
/// 親ディレクトリや一時ファイルを作成できない場合、または出力先を置き換えられない場合に返す。
pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic file destination must have a file name",
        )
    })?;
    let temporary_path = parent.join(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        Uuid::now_v7()
    ));
    let result = (|| {
        fs::write(&temporary_path, contents)?;
        fs::rename(&temporary_path, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }

    result
}
