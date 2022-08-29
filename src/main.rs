use std::cell::RefCell;
use std::fs;
use std::path;

use hp::ParsedArguments;
use hp::{Parser, Template as HpTemplate};

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

/// Parse arguments and run the application.
fn run(args: ParsedArguments) -> Result {
    let mut derfile: Option<Derfile> = None;
    let mut config: Config;
    config = Config::load_default()?;

    if args.has("--debug") {
        DEBUG.with(|v| *v.borrow_mut() = true);
    }

    if let Some(config_arg) = args.get("-c") {
        let config_path = &config_arg.values()[0];
        if let Ok(conf) = Config::load(&config_path) {
            config = conf;
            if debug() {
                println!("[\x1b[32mINFO\x1b[0m] Config file looks like: {}", config)
            }
        } else {
            config = Config::load_default()?;
        }
    }

    if let Some(derfile_arg) = args.get("-f") {
        // Get an absolute path to derfile.
        let open_derfile =
            fs::read_to_string(&path::Path::new(&derfile_arg.values()[0]).canonicalize()?)?;
        derfile = Some(derfile::Derfile::load_derfile(
            open_derfile,
            &path::Path::new(&derfile_arg.values()[0]).canonicalize()?,
            &config,
        )?);
    }
    if args.has("-a") {
        // Apply template files according to derfile rules.
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

    Ok(())
}

fn main() -> Result {
    let mut parser = Parser::new()
        .with_author("zir <kamo.bavmesa@gmail.com>")
        .with_description("Dotfile management tool.")
        .with_program_name("der");

    parser.add_template(
        HpTemplate::new()
            .matches("-f")
            .matches("--file")
            .with_help("Use a specified derfile.")
            .number_of_values(1)
            .optional_values(false),
    );
    parser.add_template(
        HpTemplate::new()
            .matches("-a")
            .matches("--apply")
            .with_help("Parse and apply a derfile."),
    );
    parser.add_template(
        HpTemplate::new()
            .matches("-p")
            .matches("--print")
            .matches("--debug")
            .with_help("Set a higher verbosity."),
    );
    parser.add_template(
        HpTemplate::new()
            .matches("-c")
            .matches("--config")
            .with_help("Specify a different than default configuration file.")
            .number_of_values(1)
            .optional_values(false),
    );

    let result = parser.parse(None);

    if let Err(e) = result {
        println!("{e}");
        return Err(e.into());
    }

    if let Err(e) = run(result.unwrap()) {
        println!("{}", e);
        return Ok(());
    }

    if DEBUG.with(|v| *v.borrow()) {
        println!("[\x1b[32mSuccess!\x1b[0m]")
    }

    Ok(())
}
