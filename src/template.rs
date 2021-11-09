use super::derfile;

pub const TEMP_START: &str = "@@";
pub const TEMP_END: &str = "@!";

use std::fs;
use std::env;
use std::path;
use std::io;

#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub path: String,
    pub final_name: String,
    pub apply_path: String,
    pub hostnames: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedTemplate(String);

impl TemplateFile {
    pub fn new(path: String, final_name: String, apply_path: String, hostnames: Vec<String>) -> Self {
        Self {
            path,
            final_name,
            apply_path,
            hostnames,
        }
    }

    pub fn parse(&self) -> Option<ParsedTemplate> {
        // [x] make sure the file even exists
        // [x] make sure there is an equal number of opening and closing template code symbols
        // [x] maybe make the actuall parsing more pretty, maybe even implement it just by removing
        // the unwanted lines? eg. the code block start and end files
        // [x] fix the bug, where code_block lines that are not valid for the current host name
        // still get included into the output file
        let mut ret = String::new();
        let hostname = env::var("HOSTNAME");
        let mut lines_to_add: Vec<(Vec<String>, usize, usize)> = Vec::new();
        if hostname.is_err() {
            eprintln!("Error when parsing template file: Unable to get the value of $HOSTNAME environment variable!");
            return None;
        }
        if !self.hostnames.contains(hostname.as_ref().unwrap()) {
            eprintln!(
                "Warning: $HOSTNAME not in hostnames for template file: {}",
                self.path
            )
        }
        if !path::Path::new(&self.path).exists() {
            return None;
        }

        let file_lines = fs::read_to_string(&self.path)
            .expect(&format!("Error: Failed to read tempalte {}", &self.path).to_string());

        // find all template code blocks
        let mut code_block_lines: Vec<(usize, String)> = Vec::new();
        for (ii, line) in file_lines.lines().enumerate() {
            if line.starts_with(TEMP_START) || line.starts_with(TEMP_END) {
                code_block_lines.push((ii, line.to_string()));
            }
        }

        // check if all blocks are closed
        let open_code_blocks_count = code_block_lines
            .iter()
            .filter(|x| x.1.starts_with(TEMP_START))
            .count();
        let closed_code_blocks_count = code_block_lines
            .iter()
            .filter(|x| x.1.starts_with(TEMP_END))
            .count();

        if open_code_blocks_count != closed_code_blocks_count {
            eprintln!("Error when parsing template file: Open template blocks don't match closed template blocks!");
            return None;
        }

        if open_code_blocks_count == 0 {
            eprintln!("No code blocks were found in file {}", self.path);
            return Some(ParsedTemplate(file_lines));
        }

        let file_lines_vec = file_lines
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        // get all the lines and their indecies for substitution in the result file
        for chunk in code_block_lines.chunks(2) {
            let code_block_start_index = chunk[0].0;
            let code_block_end_index = chunk[1].0;

            let current_code_block =
                file_lines_vec[code_block_start_index + 1..code_block_end_index].to_vec();
            let code_block_first_line = &chunk[0].1;
            let code_block_first_line_wo_prefix =
                code_block_first_line.strip_prefix(TEMP_START)?.to_string();
            // split line by `,` to get a list of hostnames on which the code block sohould be
            // applied
            let possible_hostnames = code_block_first_line_wo_prefix
                .split(",")
                .into_iter()
                .map(|x| x.trim())
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .to_vec();

            for each in possible_hostnames {
                if &each == hostname.as_ref().unwrap() {
                    lines_to_add.push((
                        current_code_block.clone(),
                        code_block_start_index,
                        code_block_end_index + 1,
                    ))
                }
            }
        }

        let mut parsed_code_blocks = Vec::new();
        for chunk in code_block_lines.chunks(2) {
            let code_block_first_line = &chunk[0].1;
            let code_block_start_index = chunk[0].0;

            let code_block_second_line = &chunk[1].1;
            let code_block_end_index = chunk[1].0;

            let possible_hostnames = code_block_first_line
                .strip_prefix(TEMP_START)?
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

            if hostnames.contains(&hostname.as_ref().unwrap()) {
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
        println!("=== new file ===");
        println!("{}", ret);
        println!("=== end file ===");

        Some(ParsedTemplate(ret))
    }

    pub fn apply(&mut self) -> io::Result<()> {
        let parsed = self.parse();
        if parsed.is_none() {
            return Ok(());
        }

        if self.apply_path.ends_with("/") {
            self.apply_path.push_str(&self.final_name);
        } else {
            self.apply_path.push('/');
            self.apply_path.push_str(&self.final_name)
        }
        let output_path = path::Path::new(&self.apply_path);
        if output_path.exists() {
            fs::write(output_path, parsed.unwrap().0)?;
        } else {
            fs::create_dir_all(output_path.parent().expect("This shouldn't fail?"))?;
            fs::write(output_path, parsed.unwrap().0)?
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
