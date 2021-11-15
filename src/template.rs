use crate::derfile;
use crate::error::*;

/// Begin and end code block symbols, these CAN be changed before compilation.
pub const TEMP_START: &str = "@@";
pub const TEMP_END: &str = "@!";

use std::env;
use std::fs;
use std::path;

/// Information needed for parsing a template file.
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Path to template file.
    pub path: String,
    /// Name of the file to be output. Example: `alacritty.yml`
    pub final_name: String,
    /// Directory to which the parsed template file should be placed: Example: `~/.config/alacritty/` 
    pub apply_path: String,
    /// Hostnames for which the template file should be parsed.
    pub hostnames: Vec<String>,
}

/// String ouput of a parsed template file.
#[derive(Debug, Clone)]
pub struct ParsedTemplate(String);

impl TemplateFile {
    /// Create a new instance of a `TemplateFile`.
    pub fn new(
        path: String,
        final_name: String,
        apply_path: String,
        hostnames: Vec<String>,
    ) -> Self {
        Self {
            path,
            final_name,
            apply_path,
            hostnames,
        }
    }

    /// Parsed a template file and output a `ParsedTemplate` struct.
    pub fn parse(&self) -> Result<ParsedTemplate> {
        // [x] make sure the file even exists
        // [x] make sure there is an equal number of opening and closing template code symbols
        // [x] maybe make the actuall parsing more pretty, maybe even implement it just by removing
        // the unwanted lines? eg. the code block start and end files
        // [x] fix the bug, where code_block lines that are not valid for the current host name
        // still get included into the output file


        // Basic stuff.
        let mut ret = String::new();
        let hostname = env::var("HOSTNAME")?;
        if !self.hostnames.contains(&hostname) {
            eprintln!(
                "Warning: $HOSTNAME not in hostnames for template file: {}",
                self.path
            )
        }
        if !path::Path::new(&self.path).exists() {
            return Err("Error parsing template file: File does not exist1".into());
        }

        let file_lines = fs::read_to_string(&self.path)
            .expect(&format!("Error: Failed to read tempalte {}", &self.path).to_string());

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
            return Err("Error when parsing template file: Open template blocks don't match closed template blocks!".into());
        }

        if open_code_blocks_count == 0 {
            eprintln!("No code blocks were found in file {}", self.path);
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

            while parsed_first_line.starts_with(" ") {
                parsed_first_line = parsed_first_line.trim_start();
            }

            let possible_hostnames = parsed_first_line
                .strip_prefix(TEMP_START)
                .unwrap()
                .split(",")
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
                good_codeblock = true
            }

            if good_codeblock {
                ret.push_str(&file_lines_vec[last_ln..*start].join("\n"));
                ret.push('\n');
                let cloned_block = &file_lines_vec[*start..=*end];
                let mut parsed_block = cloned_block
                    .iter()
                    .filter(|x| x != &start_line && x != &end_line)
                    .map(|x| x.clone())
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
                last_ln = *end + 1
            }
            if ii == parsed_code_blocks.len() - 1 {
                ret.push_str(&file_lines_vec[*end + 1..].join("\n"));
            }
        }

        Ok(ParsedTemplate(ret))
    }

    /// Write a parsed template file to disk.
    pub fn apply(&mut self) -> Result {
        let parsed = self.parse()?;

        if self.apply_path.ends_with("/") {
            self.apply_path.push_str(&self.final_name);
        } else {
            self.apply_path.push('/');
            self.apply_path.push_str(&self.final_name)
        }
        let output_path = path::Path::new(&self.apply_path);
        println!("{:#?}", output_path);
        if output_path.exists() {
            fs::write(output_path, parsed.0).unwrap();
        } else {
            fs::create_dir_all(output_path.parent().expect("This shouldn't fail?")).unwrap();
            fs::write(output_path, parsed.0).unwrap()
        }

        Ok(())
    }
}

impl From<derfile::Template> for TemplateFile {
    fn from(other: derfile::Template) -> Self {
        Self {
            path: other.name.clone(),
            final_name: other.final_name.clone(),
            apply_path: other.apply_path.clone(),
            hostnames: other.hostnames.clone(),
        }
    }
}
