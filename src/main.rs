use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;
use std::process;

mod error;
mod derfile;
mod template;

use derfile::*;
use template::*;
use error::*;

type Args = Vec<Arg>;

enum Arg {
    Help,
    Derfile(String),
    Debug(Vec<String>),
    Apply,
}


fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for qucik multisystem application of dotfiles, with template supporting.\n");
    println!("-a --apply PATH                                           Apply using  specified path to derfile.");
    println!("-f --file  PATH                                           Use a specified derfile.");
    println!("-d --debug PATH FINAL_NAME APPLY_PATH [HOSTNAMES]         Use a specified derfile.");
    println!("-h --help  PATH                                           Show this help message.")
}


fn parse_args(args: Vec<String>) -> Args {
    let mut ret = Vec::new();

    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => ret.push(Arg::Help),
            "-f" | "--file" => ret.push(Arg::Derfile(args[i + 1].clone())),
            "-d" | "--debug" => ret.push(Arg::Debug(args[i + 1..].to_vec())),
            "-a" | "--apply" => ret.push(Arg::Apply),
            _ => (),
        }
    }

    ret
}

fn execute_code(command: String) -> Result<String> {
    let vars: HashMap<String, String> = env::vars().collect();
    if command.contains(" ") {
        let split: Vec<&str> = command.split(" ").collect();
        let cmd = split[0];
        let args = &split[1..];

        let output = process::Command::new(cmd).args(args).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect("ERROR: Unable to convert command output to string");
        return Ok(str.to_string().trim().to_string());
    } else {
        let output = process::Command::new(command).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect("ERROR: Unable to convert command output to string.");
        return Ok(str.to_string().trim().to_string());
    }
}

// TODO:
//  [x] templates
//  [x] variables ->
//  [x] variable from code execution?
//  [ ] find out what's wrong with it, it for example can't access environmental variables
fn load_derfile(path: &path::Path) -> Result<Derfile> {
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

fn run(args: Args) -> Result {
    let mut derfile: Option<Derfile> = None;
    for arg in args {
        match arg {
            Arg::Derfile(file) => {
                derfile = Some(load_derfile(path::Path::new(&file))?);
            }
            Arg::Apply => {
                let derfile_default_path = path::Path::new("derfile");

                if !derfile_default_path.exists() {
                    return Err("Error: No derfile path specified or present!".into());
                }

                if derfile.is_none() {
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
            Arg::Debug(parse_args) => {
                // [x] TODO: test this please! [TESTING DONE]
                let hostnames = parse_args[3..].to_vec();
                let mut template_config = TemplateFile::new(
                    parse_args[0].clone(),
                    parse_args[1].clone(),
                    parse_args[2].clone(),
                    hostnames,
                );
                template_config.apply()?
            }
            Arg::Help => {
                help_function();
            }
        }
    }

    Ok(())
}

fn main() -> Result {
    let args: Vec<String> = env::args().collect();
    let parsed_args = parse_args(args);
    run(parsed_args)?;

    Ok(())
}
