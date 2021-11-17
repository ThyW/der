use std::path::{Component, Path, PathBuf};

/// Normalize a path without resovling symlinks and without need the path to exist! Found in
/// `cargo` [source code](https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61)
pub fn normalize_path(path: &Path) -> PathBuf {
    let path_components = path.components();
    let mut ret = PathBuf::new();

    for component in path_components {
        match component {
            Component::Prefix(..) => (),
            Component::RootDir => ret.push(component),
            Component::CurDir => (),
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(normal_component) => ret.push(normal_component),
        }
    }

    ret
}
