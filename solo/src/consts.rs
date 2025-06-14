use std::{
    env::current_exe,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;

lazy_static! {
    /// The path to the directory where the executable is located.
    pub static ref EXE_DIR: PathBuf = {
        current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap()
    };

    /// The name of the executable, without the path.
    pub static ref EXE_NAME: String = {
        current_exe()
            .ok()
            .and_then(|path| {
                path.file_name().map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "solo".into())
    };
}
