use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::*;
use crate::DEBUG;

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
        return Err("Not a directory!".to_string().into());
    }

    Ok(ret)
}

pub fn remove_template_ext_or_dir<S: AsRef<str>, P: AsRef<Path>>(
    haystack: &P,
    needles: &[S],
) -> String {
    let path = haystack.as_ref().to_path_buf();
    // HACK: fix this pls
    let final_component = path.components().last().unwrap();
    let final_component_string = final_component.as_os_str().to_str().unwrap().to_string();

    if path.is_dir() {
        return final_component_string;
    }

    let ext = path.extension().unwrap().to_str().unwrap();

    for each in needles {
        if each.as_ref() == <&str>::clone(&ext) {
            return final_component_string.replace(&format!(".{}", ext), "");
        }
    }
    final_component_string
}

pub fn debug() -> bool {
    DEBUG.with(|v| *v.borrow())
}

pub fn execute_code<S: AsRef<str>>(command: S) -> Result<String> {
    // Get a list of all environmental args.
    let vars: HashMap<String, String> = env::vars().collect();
    // Split the command into its components.
    let command = command.as_ref();
    if command.contains(' ') {
        let split: Vec<&str> = command.split(' ').collect();
        let cmd = split[0];
        let args = &split[1..];

        let output = Command::new(cmd).args(args).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.unwrap_or_else(|_| {
            panic!(
                "ERROR: Unable to convert command output to string! [Error value: {}]",
                &command
            )
        });
        return Ok(str.trim().to_string());
    } else {
        let output = Command::new(&command).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.unwrap_or_else(|_| {
            panic!(
                "ERROR: Unable to convert command output to string! [Error value: {}]",
                &command
            )
        });
        return Ok(str.trim().to_string());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dir_test() {
        assert!(super::visit_directories(&String::from("./test")).is_ok());
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

    #[test]
    fn test_execute_code() {
        assert!(super::execute_code("hostnamectl hostname").is_ok())
    }

    #[test]
    fn test_hostname() {
        assert_eq!(
            super::execute_code("hostnamectl hostname").unwrap(),
            super::execute_code("cat /etc/hostname").unwrap()
        )
    }
}
