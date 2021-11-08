use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::process;

const TEMPLATE_LEFT: &str = "[";
const TEMPLATE_RIGHT: &str = "]";
const CODE_SEP: &str = "`";
const VAR_PREF: &str = "$";
const CODE_KEYWORDS: [&str; 1] = ["env"];
const TEMP_START: &str = "@@";
const TEMP_END: &str = "@";

type Args = Vec<Arg>;

enum Arg {
    Help,
    Derfile(String),
    Parse(Vec<String>),
    Apply,
}

#[derive(Debug, Clone, Default)]
struct Template {
    name: String,
    final_name: String,
    hostnames: Vec<String>,
    apply_path: String,
}

#[derive(Debug, Clone, Default)]
struct Variable {
    _name: String,
    value: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct Derfile {
    templates: HashMap<String, Template>,
    vars: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
struct TemplateFile {
    path: String,
    final_name: String,
    apply_path: String,
    hostnames: Vec<String>,
}

#[derive(Debug, Clone)]
struct ParsedTemplate(String);

impl TemplateFile {
    fn new(
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

    fn parse(&self) -> Option<ParsedTemplate> {
        // [x] make sure the file even exists
        // [x] make sure there is an equal number of opening and closing template code symbols
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
                code_block_lines.push((ii, line.to_string()))
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

        // get all the lines and their indecies for substitution in the result file
        for (ii, each) in code_block_lines.iter().enumerate() {
            let code_block_start_index = each.0;
            let code_block_end_index = code_block_lines[ii + 1].0;

            let current_code_block = file_lines
                .lines()
                .map(ToString::to_string)
                .collect::<Vec<String>>()[code_block_start_index..code_block_end_index]
                .to_vec();
            let code_block_first_line = &current_code_block[0];
            let code_block_first_line_wo_prefix =
                code_block_first_line.strip_suffix("@@")?.to_string();
            // split line by `,` to get a list of hostnames on which we want to apply
            let possible_hostnames = code_block_first_line_wo_prefix
                .split(",")
                .into_iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .to_vec();

            for each in possible_hostnames {
                if &each == hostname.as_ref().unwrap() {
                    lines_to_add.push((
                        current_code_block[1..current_code_block.len() - 2].to_vec(),
                        code_block_start_index,
                        code_block_end_index,
                    ))
                }
            }
        }

        let mut line_nr_before_code_block = 0;
        let file_lines_vec = file_lines
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        for parsed_block in lines_to_add {
            let (lines, begin, end) = parsed_block;
            ret.push_str(&file_lines_vec[line_nr_before_code_block..begin].join("\n"));
            ret.push_str(&lines.join("\n"));
            line_nr_before_code_block = end
        }

        Some(ParsedTemplate(ret))
    }

    fn apply(&mut self) -> io::Result<()> {
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

impl From<Template> for TemplateFile {
    fn from(other: Template) -> Self {
        Self {
            path: other.name.clone(),
            final_name: other.final_name.clone(),
            apply_path: other.apply_path.clone(),
            hostnames: other.hostnames.clone(),
        }
    }
}

fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for qucik multisystem application of dotfiles, with template supporting.\n");
    println!("-a --apply       apply dotfiles to a path.");
    println!("-f --file PATH        use a specified derfile.");
    println!("-h --help PATH        show this help message.")
}

impl Template {
    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn set_final_name(&mut self, final_name: String) {
        self.final_name = final_name
    }

    fn add_hostname(&mut self, hostname: String) {
        self.hostnames.push(hostname)
    }

    fn set_apply_path(&mut self, apply_path: String) {
        self.apply_path = apply_path
    }
}

impl Variable {
    fn new(_name: String, value: Vec<String>) -> Self {
        Self { _name, value }
    }
}

impl Derfile {
    fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    fn get_template(&mut self, name: &String) -> Option<&mut Template> {
        self.templates.get_mut(name)
    }

    fn add_var(&mut self, name: String, value: Vec<String>) {
        self.vars.insert(name.clone(), Variable::new(name, value));
    }

    fn get_var(&mut self, name: &String) -> Option<&mut Variable> {
        self.vars.get_mut(name)
    }

    fn parse(&mut self) -> Self {
        let mut derfile: Derfile = Default::default();
        let mut self_clone = self.clone();
        for (template_name, template) in self.templates.iter() {
            derfile.add_template(template_name.clone());
            let mut temp = derfile.get_template(&template_name).unwrap();
            temp.name = template_name.to_string();
            if template.final_name.starts_with(VAR_PREF) {
                let variable_name = template
                    .final_name
                    .strip_prefix(VAR_PREF)
                    .unwrap()
                    .to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut temp = derfile.get_template(&template.name).unwrap();
                    temp.final_name = variable.value[0].clone(); // only take the fist value, sicne we only accept only one final file name
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.final_name = template.final_name.clone();
            }
            if template.apply_path.starts_with(VAR_PREF) {
                let variable_name = template
                    .apply_path
                    .strip_prefix(VAR_PREF)
                    .unwrap()
                    .to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut template = derfile.get_template(&template.name).unwrap();
                    template.apply_path = variable.value[0].clone(); // only take the first value, since we only accpet one apply path now
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.apply_path = template.apply_path.clone()
            }
            let mut hostname_clone: Vec<String> = Vec::new();
            for hostname_entry in template.hostnames.iter() {
                if hostname_entry.starts_with(VAR_PREF) {
                    if let Some(variable) = self_clone
                        .get_var(&hostname_entry.strip_prefix(VAR_PREF).unwrap().to_string())
                    {
                        let mut variable_value = variable.value.clone();
                        hostname_clone.append(&mut variable_value);
                    }
                } else {
                    hostname_clone.push(hostname_entry.to_string())
                }
            }

            let template = derfile.get_template(&template.name).unwrap();
            template.hostnames = hostname_clone
        }
        derfile.vars = self.vars.clone();

        derfile
    }
}

fn parse_args(args: Vec<String>) -> Args {
    let mut ret = Vec::new();

    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => ret.push(Arg::Help),
            "-f" | "--file" => ret.push(Arg::Derfile(args[i + 1].clone())),
            "-p" | "--parse" => ret.push(Arg::Parse(args[i..i + 3].to_vec())),
            "-a" | "--apply" => ret.push(Arg::Apply),
            _ => (),
        }
    }

    ret
}

fn execute_code(command: String) -> Option<String> {
    let output: Option<String>;
    let vars: HashMap<String, String> = env::vars().collect();
    if command.contains(" ") {
        let split: Vec<&str> = command.split(" ").collect();
        let cmd = split[0];
        let args = &split[1..];

        let process = process::Command::new(cmd).args(args).envs(&vars).output();

        if let Ok(out) = process {
            let str = std::str::from_utf8(&out.stdout);
            let str = str.expect("ERROR: Unable to convert command output to string");
            output = Some(str.to_string().trim().to_string())
        } else {
            eprintln!("ERROR: code block exited with exited with an error.");
            output = Some("".to_string())
        }
    } else {
        let process = process::Command::new(command).envs(&vars).output();
        if let Ok(out) = process {
            let str = std::str::from_utf8(&out.stdout);
            let str = str.expect("ERROR: Unable to convert command output to string.");
            output = Some(str.to_string().trim().to_string())
        } else {
            eprintln!("ERROR: code block exited with exited with an error.");
            output = Some("".to_string())
        }
    }

    output
}

// TODO:
//  [x] templates
//  [x] variables ->
//      [x] variable from code execution?
//      [ ] find out what's wrong with it, it for example can't access environmental variables
fn load_derfile(path: &path::Path) -> io::Result<Derfile> {
    let buffer = fs::read_to_string(path)?;
    let mut derfile: Derfile = Default::default();
    let lines = buffer.lines();

    // list of template definitions(lines that start with "[" and end with "]")
    let template_indecies: Vec<usize> = lines
        .clone()
        .enumerate()
        .filter(|line| {
            line.1.trim().starts_with(TEMPLATE_LEFT) && line.1.trim().ends_with(TEMPLATE_RIGHT)
        })
        .map(|each| return each.0)
        .collect();

    let var_lines: Vec<String> = lines
        .clone()
        .enumerate()
        .filter(|line| line.1.trim().starts_with(VAR_PREF))
        .map(|each| return each.1.to_string())
        .collect();

    // TODO
    // [x] variables from shell code
    // [x] stuff such as "echo $PATH" does not work
    // E.i. instead of returning the value
    // of environmental variable $PATH(which could look like:
    // /home/user/.local/bin/:/usr/bin/:/usr/local/bin/ etc.) we just get the actual string
    // "$PATH".
    // I have found no solution for this, you might look into this and submit a PR and make me feel
    // like a dumb ass(which would be highly appreciated). But if this worked, I would be delighted.
    // What we do instead, is have a special keyword for returning environmental variables. So
    // something like this: "env`$PATH`" could return the actual value of our environmental
    // variable.
    for line in var_lines.iter() {
        if line.contains("=") {
            let split = line.split_at(line.find("=").unwrap());
            let name = split.0.trim().strip_prefix(VAR_PREF).unwrap().to_string();
            let mut value: Vec<String> = Vec::new();
            let right_side = split
                .1
                .trim()
                .to_string()
                .strip_prefix("=")
                .unwrap()
                .trim()
                .to_string();

            if right_side.starts_with(CODE_SEP) {
                if let Some(index) = right_side.find(CODE_SEP) {
                    let content = right_side[index + 1..right_side.len() - 1].to_string();
                    value =
                        vec![execute_code(content.clone())
                            .expect("Error: unablet to parse code block.")];
                }
            } else if right_side.contains(",") {
                let split: Vec<String> = right_side
                    .split(",")
                    .map(|x| x.trim().to_string())
                    .collect();
                value = split
            } else {
                value = vec![right_side.clone()]
            }

            // Special keywords are defined in const CODE_KEYWORDS and each of them is manually
            // implemented here. So far, there's only "env", which is used to retrieve environmental
            // variables
            for each in CODE_KEYWORDS {
                if right_side.starts_with(each) {
                    match each {
                        "env" => {
                            if let Some(index) = right_side.find(CODE_SEP) {
                                let env_variable = &right_side[index + 1..right_side.len() - 1];
                                println!("{}", env_variable);

                                if let Ok(env_output) = env::var(env_variable) {
                                    value = vec![env_output]
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            derfile.add_var(name, value)
        }
    }

    let lines: Vec<String> = lines.clone().map(|x| x.to_string()).collect();
    for (ii, index) in template_indecies.iter().enumerate() {
        let template_lines: Vec<String>;

        // TODO: inefficient? shouldn't clone on each iteration,
        // but maybe its okay since its dropped before the next iteration?
        if template_indecies.len() == 1 {
            template_lines = lines.clone().drain(index..).collect();
        } else if ii == template_indecies.len() - 1 {
            template_lines = lines.clone().drain(index..).collect();
        } else {
            template_lines = lines
                .clone()
                .drain(index..&template_indecies[ii + 1])
                .collect();
        }

        let template_name: String = template_lines[0][1..template_lines[0].len() - 1].to_string();
        for line in template_lines.iter() {
            if line.starts_with(TEMPLATE_LEFT) && line.ends_with(TEMPLATE_RIGHT) {
                derfile.add_template(template_name.clone());
                if let Some(template) = derfile.get_template(&template_name) {
                    template.set_name(template_name.clone())
                }
            }

            if line.is_empty() {
                continue;
            }

            if line.contains("=") {
                let split = line.split_at(line.find("=").unwrap());
                match split.0.trim() {
                    "final_name" => {
                        if let Some(table) = derfile.get_template(&template_name) {
                            table.set_final_name(
                                split
                                    .1
                                    .to_string()
                                    .strip_prefix("=")
                                    .unwrap()
                                    .to_string()
                                    .trim()
                                    .to_string(),
                            )
                        }
                    }
                    "hostnames" => {
                        if let Some(table) = derfile.get_template(&template_name) {
                            if split.1.contains(",") {
                                let value_list: Vec<String> =
                                    split.1.split(',').map(|x| x.to_string()).collect();
                                for each in value_list {
                                    if let Some(s) = each.strip_prefix("=") {
                                        table.add_hostname(s.to_string().trim().to_string())
                                    } else {
                                        table.add_hostname(each.trim().to_string())
                                    }
                                }
                            } else {
                                table.add_hostname(
                                    split
                                        .1
                                        .to_string()
                                        .strip_prefix("=")
                                        .unwrap()
                                        .to_string()
                                        .trim()
                                        .to_string(),
                                )
                            }
                        }
                    }
                    "apply_path" => {
                        if let Some(table) = derfile.get_template(&template_name) {
                            table.set_apply_path(
                                split
                                    .1
                                    .to_string()
                                    .strip_prefix("=")
                                    .unwrap()
                                    .to_string()
                                    .trim()
                                    .to_string(),
                            )
                        }
                    }
                    some => {
                        if some.starts_with(VAR_PREF) {
                            continue;
                        } else {
                            eprintln!("WARN: {} is not a valid template filed!", split.0)
                        }
                    }
                }
            }
        }
    }

    Ok(derfile.parse())
}

fn run(args: Args) -> io::Result<()> {
    let mut derfile: Option<Derfile> = None;
    for arg in args {
        match arg {
            Arg::Derfile(file) => {
                let derfile = Some(load_derfile(path::Path::new(&file)).unwrap());
                println!("{:#?}", derfile.as_ref().unwrap());
            }
            Arg::Apply => {
                // TODO:
                // we want to take a derfile and parse it
                // then parse all templates and output them to their respective apply paths
                // these apply paths should be attempted to be created if they do not exist
                let derfile_default_path = path::Path::new("./derfile");

                if !derfile_default_path.exists() {
                    return Ok(());
                }

                if !derfile.is_none() {
                    derfile = Some(load_derfile(derfile_default_path)?);
                }

                let mut templates = derfile
                    .clone()
                    .unwrap()
                    .templates
                    .values()
                    .map(|t| t.clone().into())
                    .collect::<Vec<TemplateFile>>();
                for each_template in templates.iter_mut() {
                    each_template.apply()?;
                }
                    
            }
            Arg::Parse(parse_args) => {
                // TODO: test this please!
                let hostnames = parse_args[3..].to_vec();
                let template_config =
                    TemplateFile::new(parse_args[0].clone(), parse_args[1].clone(), parse_args[2].clone(), hostnames);
                if let Some(temp) = template_config.parse() {
                    println!("{:#?}", temp)
                }
            }
            Arg::Help => {
                help_function();
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let parsed_args = parse_args(args);
    run(parsed_args)?;

    Ok(())
}
