use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::*;

/// Normalize a path without resovling symlinks and without need the path to exist! Found in
/// `cargo` [source code](https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61)
pub fn normalize_path<P: AsRef<Path> + ?Sized>(path: &P) -> PathBuf {
    let path_components = path.as_ref().components();
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

#[allow(unused)]
pub fn visit_directories<P: AsRef<Path>>(path: &P) -> Result<Vec<(PathBuf, fs::Metadata)>> {
    let mut ret: Vec<(PathBuf, fs::Metadata)> = Vec::new();

    for entry_result in path.as_ref().read_dir()? {
        let entry = entry_result?;
        let entry_path = entry.path();
        match entry_path.is_dir() {
            true => {
                ret.append(&mut visit_directories(&entry_path)?);
                ret.push((entry_path.to_path_buf(), entry.metadata()?))
            }
            false => {
                ret.push((entry_path.to_path_buf(), entry.metadata()?));
            }
        }
    }

    Ok(ret)
}

pub fn list_dir<P: AsRef<Path>>(path: &P) -> Result<Vec<fs::DirEntry>> {
    let mut ret: Vec<fs::DirEntry> = Vec::new();
    if path.as_ref().is_dir() {
        for each in path.as_ref().read_dir()? {
            ret.push(each?)
        }
    } else {
        return Err("Not a directory!".into());
    }

    Ok(ret)
}

pub fn remove_template_ext_or_dir<S: AsRef<str>, P: AsRef<Path>>(
    haystack: &P,
    needles: &Vec<S>,
) -> String {
    let path = haystack.as_ref().to_path_buf();
    // HACK: fix this pls
    let final_component = path.components().last().unwrap();
    let final_component_string = final_component
        .clone()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    if path.is_dir() {
        return final_component_string;
    }

    let ext = path.extension().unwrap().to_str().unwrap();

    for each in needles {
        if each.as_ref() == ext.clone() {
            return final_component_string.replace(&format!(".{}", ext), "");
        }
    }
    return final_component_string;
}

#[cfg(test)]
mod tests {
    use crate::{error::*, utils::visit_directories};
    #[test]
    fn dir_test() -> Result {
        println!("{:#?}", visit_directories(&String::from("./owntest"))?);

        Ok(())
    }

    #[test]
    fn normalize_path_test() {
        assert_eq!(
            super::normalize_path(&String::from("/home/test/../../usr"))
                .as_path()
                .to_str()
                .unwrap(),
            "/usr"
        )
    }
}
