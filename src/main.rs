use std::cell::RefCell;
use std::env;
use std::fs;
use std::path;
use std::process::exit;

mod config;
mod derfile;
mod error;
mod template;
mod utils;

use config::*;
use derfile::*;
use error::*;
use template::*;
use utils::debug;
// use utils::execute_code;

// Global variable for debugging
thread_local! {static DEBUG: RefCell<bool> = RefCell::new(false)}

/// Wrapper type for a list of parsed command line arguments.
type Args = Vec<Arg>;

/// All possible command line arguments.
#[derive(Debug)]
enum Arg {
    Help,
    Derfile(String),
    Apply,
    Print,
    Config(String),
}

/// Help fucntion to be diplayed when the `-h` or `--help` flags are passed.
fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for quick multisystem application of dotfiles, with template and multi-version support.\n");
    println!("-c --config PATH\t\t\tUse a specific config file.");
    println!("-a --apply      \t\t\tAttempt to apply a derfile.");
    println!("-f --file PATH  \t\t\tUse a specified derfile.");
    println!("-h --help       \t\t\tShow this help message.");
    println!("-p --print      \t\t\tPrint status messages and debug info.")
}

/// Get a list of passed in command line arguments.
fn parse_args(args: Vec<String>) -> Args {
    // return a vector of Args
    let mut ret = Vec::new();

    // go through command line arguments and add them to the return vector
    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => ret.push(Arg::Help),
            "-f" | "--file" => {
                if i + 1 != args.len() {
                    if !args[i + 1].starts_with('-') {
                        ret.push(Arg::Derfile(args[i + 1].clone()))
                    } else {
                        println!("[ERROR] Missing arguments for --file!");
                        exit(1)
                    }
                } else {
                    println!("[ERROR] Missing arguments for --file!");
                    exit(1)
                }
            }
            "-a" | "--apply" => ret.push(Arg::Apply),
            "-p" | "--print" => ret.push(Arg::Print),
            "-c" | "--config" => {
                if i + 1 != args.len() {
                    if !args[i + 1].starts_with('-') {
                        ret.push(Arg::Config(args[i + 1].clone()))
                    } else {
                        println!("[ERROR] Missing arguments for --config!");
                        exit(1)
                    }
                } else {
                    println!("[ERROR] Missing arguments for --config!");
                    exit(1)
                }
            }
            _ => (),
        }
    }

    // print out help, if no other argument is passed
    if ret.is_empty() {
        if !args.is_empty() {
            println!("[ERROR] Unrecognized command line arguments!")
        }
        ret.push(Arg::Help)
    }

    ret
}

/// Parse arguments and run the application.
fn run(args: Args) -> Result {
    let mut derfile: Option<Derfile> = None;
    let mut config: Config;
    config = Config::load_default()?;
    for arg in args {
        match arg {
            Arg::Config(config_path) => {
                if let Ok(conf) = Config::load(&config_path) {
                    config = conf;
                    if debug() {
                        println!("[\x1b[32mINFO\x1b[0m] Config file looks like: {}", config)
                    }
                } else {
                    config = Config::load_default()?;
                }
            }
            // Specifiy a derfile to be used.
            Arg::Derfile(file) => {
                // Get an absolute path to derfile.
                let open_derfile = fs::read_to_string(&path::Path::new(&file).canonicalize()?)?;
                derfile = Some(derfile::Derfile::load_derfile(
                    open_derfile,
                    &path::Path::new(&file).canonicalize()?,
                    &config,
                )?);
            }

            // Apply template files according to derfile rules.
            Arg::Apply => {
                let derfile_default_path = path::Path::new("./derfile").canonicalize();

                if derfile_default_path.is_err() && derfile.clone().is_none() {
                    return Err("No derfile path specified or present!".to_string().into());
                }

                if derfile.is_none() {
                    let derfile_default_path = &derfile_default_path?;
                    let loaded_derfile = fs::read_to_string(&derfile_default_path)?;
                    derfile = Some(derfile::Derfile::load_derfile(
                        loaded_derfile,
                        derfile_default_path,
                        &config,
                    )?);
                }

                let template_structures: Vec<Template> = derfile
                    .clone()
                    .unwrap()
                    .templates
                    .values()
                    .map(Clone::clone)
                    .collect();
                let vecs = recursive_build(template_structures)?;
                for structure in vecs {
                    if let TemplateStructure::File(mut f) = structure {
                        if debug() {
                            println!("[\x1b[32mINFO\x1b[0m] Applying: {}", f.0.path)
                        }
                        f.apply()?;
                        if debug() {
                            println!("[\x1b[32mINFO\x1b[0m] Done!");
                        }
                    }
                }
            }
            Arg::Print => DEBUG.with(|v| *v.borrow_mut() = true),
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

    if let Err(e) = run(parsed_args) {
        println!("{}", e);
        return Ok(());
    }

    if DEBUG.with(|v| *v.borrow()) {
        println!("\x1b[32mSuccess!\x1b[0m]")
    }

    Ok(())
}
