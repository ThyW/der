use crate::derfile;
use crate::error::*;
use crate::utils::*;
use std::fs;
use std::path;

/// After derfile parsing is done, and the `-a` flag is passed, a list of all
/// `TemplateStrucutre`s will be created. These all basically boil down to a list of
/// `TemplateFile`s, which are then applied, so according to specifications from the derfile, these
/// template files will all be placed onto their apply paths, as specified in the derfile.
///
/// The whole process starts with going over the list of all `Template`s from a derfile and then
/// creating template structures from them. So if a template specifies a `TemplateFile` a
/// `TemplateStructure::File` is constructed and placed into the `TemplateStrucutre`s vector and if
/// a directory is encountered a recursive process will take place, where each directory is parsed
/// into `TemplateStructure::Directory` and all its children are stored inside it as a list of
/// `TemplateFile`s.

/// Begin and end code block symbols, these CAN be changed before compilation.
pub const TEMP_START: &str = "@@";
pub const TEMP_END: &str = "@!";

/// This type alias represents a vector of `TemplateStructure`s.
pub type TemplateStructures = Vec<TemplateStructure>;

/// Information needed for parsing a template file.
#[derive(Debug, Clone)]
pub struct TemplateFile(pub TemplateSettings, pub Option<String>);

/// Information need for parsing either a template file or a template directory.
#[derive(Debug, Clone)]
pub struct TemplateSettings {
    /// Path to template file.
    pub path: String,
    /// Name of the file to be output. Example: `alacritty.yml`
    pub final_name: String,
    /// Directory to which the parsed template file should be placed: Example: `~/.config/alacritty/`
    pub apply_path: String,
    /// Hostnames for which the template file should be parsed.
    pub hostnames: Vec<String>,
    /// If this strcutre is a directory, should all its files be parsed?
    pub parse_files: bool,
    /// Extension to look for within this directory,
    pub extensions: Vec<String>,
    /// Should parse files recursively in all its subdirectories?
    pub recursive: bool,
}

/// A template strucutre is either a template file or a template directory, which can then hold
/// other template files.
#[derive(Debug, Clone)]
pub enum TemplateStructure {
    File(TemplateFile),
    Directory(TemplateDirectory),
}

#[derive(Debug, Clone)]
/// Information, which represents a template directory. This directory can consists of multiple
/// template files.
pub struct TemplateDirectory {
    pub settings: TemplateSettings,
}

/// String ouput of a parsed template file.
#[derive(Debug, Clone)]
pub struct ParsedTemplate(String);

impl TemplateFile {
    /// Create a new instance of a `TemplateFile`.
    pub fn new(ts: TemplateSettings, s: Option<String>) -> Self {
        Self(ts, s)
    }

    pub fn read(&mut self) -> Result {
        // this for some reason is pretty weird.
        let result = fs::read_to_string(&self.0.path);

        if let Err(read_error) = result {
            eprintln!(
                "[WARN] Template file: {:?} could not be read. This error was returned",
                read_error
            );
            Err(Error::Io(read_error))
        } else {
            let contents = result.unwrap();
            self.1 = Some(contents);
            Ok(())
        }
    }

    /// Parsed a template file and output a `ParsedTemplate` struct.
    pub fn parse(&mut self) -> Result<ParsedTemplate> {
        // [x] make sure the file even exists
        // [x] make sure there is an equal number of opening and closing template code symbols
        // [x] maybe make the actuall parsing more pretty, maybe even implement it just by removing
        // the unwanted lines? eg. the code block start and end files
        // [x] fix the bug, where code_block lines that are not valid for the current host name
        // still get included into the output file

        // Basic stuff.
        let mut ret = String::new();
        let hostname = execute_code("hostnamectl hostname")?;
        if !self.0.hostnames.contains(&hostname) && debug() {
            eprintln!(
                "[\x1b[33mWARN\x1b[0m] $HOSTNAME not in hostnames for template file: {}",
                self.0.path
            )
        }
        if !path::Path::new(&self.0.path).exists() {
            return Err("Error parsing template file: File does not exist1"
                .to_string()
                .into());
        }
        let file_lines: String;

        // this magic to enable testing
        if let Some(fls) = &self.1 {
            file_lines = fls.to_string();
        } else {
            self.read()?;
            file_lines = self.1.as_ref().unwrap().to_string()
        }

        // Find all template code blocks.
        let mut code_block_lines: Vec<(usize, String)> = Vec::new();
        for (ii, line) in file_lines.lines().enumerate() {
            if line.contains(TEMP_START) || line.contains(TEMP_END) {
                code_block_lines.push((ii, line.to_string()));
            }
        }

        // Check if all blocks are closed.
        let open_code_blocks_count = code_block_lines
            .iter()
            .filter(|x| x.1.contains(TEMP_START))
            .count();
        let closed_code_blocks_count = code_block_lines
            .iter()
            .filter(|x| x.1.contains(TEMP_END))
            .count();

        if open_code_blocks_count != closed_code_blocks_count {
            return Err("Error when parsing template file: Open template blocks don't match closed template blocks!".to_string().into());
        }

        if open_code_blocks_count == 0 {
            if debug() {
                eprintln!(
                    "[\x1b[33mWARN\x1b[0m] No code blocks were found in file {}",
                    self.0.path
                );
            }
            return Ok(ParsedTemplate(file_lines));
        }

        let file_lines_vec = file_lines
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        let mut parsed_code_blocks = Vec::new();
        for chunk in code_block_lines.chunks(2) {
            let code_block_first_line = &chunk[0].1;
            let code_block_start_index = chunk[0].0;

            let code_block_second_line = &chunk[1].1;
            let code_block_end_index = chunk[1].0;

            let parsed_first_line = code_block_first_line.clone();
            let mut parsed_first_line = parsed_first_line.as_str();

            while parsed_first_line.starts_with(' ') {
                parsed_first_line = parsed_first_line.trim_start();
            }

            let possible_hostnames = parsed_first_line
                .strip_prefix(TEMP_START)
                .unwrap()
                .split(',')
                .into_iter()
                .map(|x| x.trim())
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .to_vec();

            parsed_code_blocks.push((
                code_block_first_line.clone(),
                code_block_start_index,
                code_block_second_line.clone(),
                code_block_end_index,
                possible_hostnames,
            ))
        }

        // last line before code block
        let mut last_ln = 0;
        for (ii, each) in parsed_code_blocks.iter().enumerate() {
            // begin line, start line number of code block, end line, end line number of
            // code_block, hostnames for this code block
            let mut good_codeblock = false;
            let (start_line, start, end_line, end, hostnames) = each;

            if hostnames.contains(&hostname) {
                good_codeblock = true;
            }

            if good_codeblock {
                ret.push_str(&file_lines_vec[last_ln..*start].join("\n"));
                ret.push('\n');
                let cloned_block = &file_lines_vec[*start..=*end];
                let mut parsed_block = cloned_block
                    .iter()
                    .filter(|x| x != &start_line && x != &end_line)
                    .cloned()
                    .collect::<Vec<String>>()
                    .to_vec();
                let mut ready_block = parsed_block
                    .iter_mut()
                    .map(|x| {
                        if x.is_empty() {
                            String::from("\n")
                        } else {
                            x.to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .to_vec();
                if ready_block.len() == 1 {
                    ready_block[0].push('\n');
                }
                ret.push_str(&ready_block.join("\n"));
                last_ln = *end + 1;
            } else {
                ret.push_str(&file_lines_vec[last_ln..*start].join("\n"));
                ret.push('\n');
                last_ln = *end + 1
            }
            if ii == parsed_code_blocks.len() - 1 {
                ret.push_str(&file_lines_vec[*end + 1..].join("\n"));
            }
        }
        ret.push('\n');

        Ok(ParsedTemplate(ret))
    }

    /// Write a parsed template file to disk.
    pub fn apply(&mut self) -> Result {
        let parsed = self.parse()?;

        if self.0.apply_path.ends_with('/') {
            self.0.apply_path.push_str(&self.0.final_name);
        } else {
            self.0.apply_path.push('/');
            self.0.apply_path.push_str(&self.0.final_name)
        }
        let output_path = path::Path::new(&self.0.apply_path);
        if debug() {
            println!("[\x1b[32mINFO\x1b[0m] Outputting to: {:#?}", output_path);
        }
        if output_path.exists() {
            fs::write(output_path, parsed.0).unwrap();
        } else {
            fs::create_dir_all(output_path.parent().expect("This shouldn't fail?")).unwrap();
            fs::write(output_path, parsed.0).unwrap()
        }

        Ok(())
    }
}

impl From<derfile::Template> for TemplateStructure {
    fn from(other: derfile::Template) -> Self {
        if !path::Path::new(&other.name).is_dir() {
            Self::File(TemplateFile::new(other.into(), None))
        } else {
            Self::Directory(TemplateDirectory::new(other.into()))
        }
    }
}

impl From<derfile::Template> for TemplateSettings {
    fn from(other: derfile::Template) -> Self {
        Self {
            path: other.name.clone(),
            final_name: other.final_name.clone(),
            apply_path: other.apply_path.clone(),
            hostnames: other.hostnames.clone(),
            extensions: other.extensions.clone(),
            parse_files: other.parse_files,
            recursive: other.recursive,
            // keep_structure: other.keep_structure.clone(),
        }
    }
}

impl TemplateDirectory {
    pub fn new(ts: TemplateSettings) -> Self {
        Self { settings: ts }
    }

    pub fn parse(&self) -> Result<Vec<TemplateStructure>> {
        let mut ret: Vec<TemplateStructure> = vec![];
        let current_dir_listed = list_dir(&self.settings.path)?;

        for current_dir_entry in current_dir_listed {
            let metadata = current_dir_entry.metadata()?;
            let mut path = current_dir_entry.path();
            let mut cloned_settings = self.settings.clone();

            if path.is_relative() {
                path = path.canonicalize()?;
            }
            cloned_settings.path = path.to_str().unwrap().to_string();
            cloned_settings.final_name =
                remove_template_ext_or_dir(&cloned_settings.path, &cloned_settings.extensions);
            let mut apply_path_path = path::PathBuf::from(&cloned_settings.apply_path);
            apply_path_path.push(&self.settings.final_name);
            cloned_settings.apply_path = apply_path_path.to_str().unwrap().to_string();

            if metadata.is_dir() {
                if !cloned_settings.recursive {
                    continue;
                } else {
                    let dir = TemplateDirectory::new(cloned_settings);

                    ret.push(TemplateStructure::Directory(dir.clone()));
                    ret.append(&mut dir.parse()?);
                }
            } else if metadata.is_file() {
                ret.push(TemplateStructure::File(TemplateFile::new(
                    cloned_settings,
                    None,
                )));
            } else {
                // FIXME
                // symlinks and other stuff is skipped too
                continue;
            }
        }

        Ok(ret)
    }
}

pub fn recursive_build(input: Vec<derfile::Template>) -> Result<TemplateStructures> {
    let mut ret: TemplateStructures = Vec::new();
    for template in input.into_iter() {
        if path::Path::new(&template.name).is_dir() {
            let settings: TemplateSettings = template.into();
            let dir = TemplateDirectory::new(settings);

            ret.push(TemplateStructure::Directory(dir.clone()));
            ret.append(&mut dir.parse()?)
        } else if path::Path::new(&template.name).is_file() {
            let settings: TemplateSettings = template.into();
            let file: TemplateFile = TemplateFile::new(settings, None);

            ret.push(TemplateStructure::File(file));
        }
        // FIXME
        // Skip symlinks for now. We should follow them later on.
        else if path::Path::new(&template.name).is_symlink() {
            continue;
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::derfile::Derfile;
    use super::execute_code;
    use super::TemplateFile;
    use crate::config::Config;
    use std::path::Path;

    #[test]
    fn test_template_file() {
        let hostname = execute_code("cat /etc/hostname").unwrap();
        let derfile_string = r"#
$host = `hostname`
$out = some/out/path/

[/etc]
final_name = name
apply_path = $out
hostnames = $host
"
        .to_string();

        let template_string = format!(
            "some stuff\n@@ {}\nmore stuff\n@!\nand even more stuff\n",
            hostname
        );

        let derfile =
            Derfile::load_derfile(derfile_string, Path::new("some_path"), &Config::default())
                .unwrap();
        let template = derfile
            .templates
            .iter()
            .filter(|t| t.0 != "[default-template]")
            .last()
            .unwrap()
            .1;
        let mut template_file = TemplateFile::new(template.clone().into(), Some(template_string));
        assert_eq!(template_file.parse().is_ok(), true);
        let output = template_file.parse().unwrap().0;
        println!("{}", output);
        assert_eq!(
            output,
            "some stuff\nmore stuff\nand even more stuff\n".to_string()
        )
    }
}
