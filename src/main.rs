#![deny(warnings)]

mod assertions;
mod error;
mod format;
mod parser;
mod schema;
mod token;
mod tokenizer;

use crate::{
    error::{from_message, listing, with_listing, Error},
    format::CodeStr,
    parser::parse,
    tokenizer::tokenize,
};
use clap::{
    App,
    AppSettings::{
        ColoredHelp, SubcommandRequiredElseHelp, UnifiedHelpMessage, VersionlessSubcommands,
    },
    Arg, Shell, SubCommand,
};
use std::{
    collections::HashSet,
    fs::read_to_string,
    io::stdout,
    path::{Path, PathBuf},
    process::exit,
};

// The program version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// The name of the program binary
const BIN_NAME: &str = "typical";

// Command-line option and subcommand names
const CHECK_SUBCOMMAND: &str = "check";
const CHECK_SUBCOMMAND_PATH_OPTION: &str = "check-path";
const SHELL_COMPLETION_SUBCOMMAND: &str = "shell-completion";
const SHELL_COMPLETION_SUBCOMMAND_SHELL_OPTION: &str = "shell-completion-shell";

// Set up the command-line interface.
fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("Typical")
        .version(VERSION)
        .version_short("v")
        .about("Typical is an interface definition language.")
        .setting(SubcommandRequiredElseHelp) // [tag:subcommand_required_else_help]
        .setting(ColoredHelp)
        .setting(UnifiedHelpMessage)
        .setting(VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name(CHECK_SUBCOMMAND)
                .about("Checks a schema")
                .arg(
                    Arg::with_name(CHECK_SUBCOMMAND_PATH_OPTION)
                        .value_name("PATH")
                        .help("Sets the path of the schema")
                        .required(true) // [tag:check_subcommand_shell_required]
                        .takes_value(true)
                        .number_of_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name(SHELL_COMPLETION_SUBCOMMAND)
                .about(
                    " \
                     Prints a shell completion script. Supports Zsh, Fish, Zsh, PowerShell, and \
                     Elvish. \
                     "
                    .trim(),
                )
                .arg(
                    Arg::with_name(SHELL_COMPLETION_SUBCOMMAND_SHELL_OPTION)
                        .value_name("SHELL")
                        .help("Bash, Fish, Zsh, PowerShell, or Elvish")
                        .required(true) // [tag:shell_completion_subcommand_shell_required]
                        .takes_value(true)
                        .number_of_values(1),
                ),
        )
}

// Check a schema.
#[allow(clippy::too_many_lines)]
fn check_schema(schema_path: &Path) -> Result<(), Error> {
    // Canonicalize the path to avoid loading the same file multiple
    // times.
    let canonical_schema_path = match schema_path.canonicalize() {
        Ok(canonical_path) => canonical_path,
        Err(error) => {
            return Err(from_message(
                &format!(
                    "Unable to canonicalize path {}.",
                    schema_path.to_string_lossy().code_str(),
                ),
                None,
                Some(error),
            ));
        }
    };
    drop(schema_path); // So we don't accidentally use this from now on

    // Initialize the "frontier" with the given path.
    let mut paths_to_load: Vec<(PathBuf, Option<(PathBuf, String)>)> =
        vec![(canonical_schema_path.clone(), None)];
    let mut visited_paths = HashSet::new();
    visited_paths.insert(canonical_schema_path);

    // Perform a depth-first traversal of the transitive dependencies.
    let mut errors = vec![];
    let mut first = true;
    while let Some((path, origin)) = paths_to_load.pop() {
        // Compute the base directory for this schema's dependencies.
        let base_dir = if let Some(base_dir) = path.parent() {
            base_dir
        } else {
            let message = format!("Path {} has no parent.", path.to_string_lossy().code_str());

            if let Some((origin_path, origin_listing)) = origin {
                errors.push(with_listing::<Error>(
                    &message,
                    Some(&origin_path),
                    &origin_listing,
                    None,
                ));
            } else {
                errors.push(from_message::<Error>(&message, None, None));
            }

            continue;
        };

        // Read the file.
        let contents = match read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                let message = format!("Unable to read file {}.", path.to_string_lossy().code_str());

                if let Some((origin_path, origin_listing)) = origin {
                    errors.push(with_listing(
                        &message,
                        Some(&origin_path),
                        &origin_listing,
                        Some(error),
                    ));
                } else {
                    errors.push(from_message(&message, None, Some(error)));
                }

                continue;
            }
        };

        // Tokenize the contents.
        let tokens = match tokenize(&path, &contents) {
            Ok(tokens) => tokens,
            Err(error) => {
                errors.extend_from_slice(&error);

                continue;
            }
        };

        // Parse the tokens.
        let schema = match parse(&path, &contents, &tokens) {
            Ok(schema) => schema,
            Err(error) => {
                errors.extend_from_slice(&error);

                continue;
            }
        };

        // Print the schema. Note that the schema's representation includes
        // a trailing empty line.
        if first {
            first = false;
        } else {
            println!();
        }

        print!("{}", schema.to_string().code_str());

        // Add the dependencies to the frontier.
        for import in &schema.imports {
            // Compute the source listing for this import for error
            // reporting.
            let origin_listing = listing(&contents, import.source_range.0, import.source_range.1);

            // Compute the import path.
            let path_to_canonicalize = base_dir.join(&import.path);

            // Canonicalize the path to avoid loading the same file multiple
            // times.
            match path_to_canonicalize.canonicalize() {
                Ok(canonical_path) => {
                    // Visit this import if it hasn't been visited already.
                    if !visited_paths.contains(&canonical_path) {
                        visited_paths.insert(import.path.to_owned());
                        paths_to_load.push((canonical_path, Some((path.clone(), origin_listing))));
                    }
                }
                Err(error) => {
                    errors.push(with_listing(
                        &format!(
                            "Unable to canonicalize path {}.",
                            path_to_canonicalize.to_string_lossy().code_str(),
                        ),
                        Some(&path),
                        &origin_listing,
                        Some(error),
                    ));
                }
            }
        }
    }

    if !first {
        println!();
    }

    // Return a success or report any errors.
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error {
            message: errors
                .iter()
                .fold(String::new(), |acc, error| {
                    format!(
                        "{}\n{}{}",
                        acc,
                        // Only render an empty line between errors here if the previous line
                        // doesn't already visually look like an empty line. See
                        // [ref:overline_u203e].
                        if acc
                            .split('\n')
                            .last()
                            .unwrap()
                            .chars()
                            .all(|c| c == ' ' || c == '\u{203e}')
                        {
                            ""
                        } else {
                            "\n"
                        },
                        error,
                    )
                })
                .trim()
                .to_owned(),
            reason: None,
        })
    }
}

// Print a shell completion script to STDOUT.
fn shell_completion(shell: &str) -> Result<(), Error> {
    // Determine which shell the user wants the shell completion for.
    let shell_variant = match shell.trim().to_lowercase().as_ref() {
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        "zsh" => Shell::Zsh,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            return Err(Error {
                message: format!(
                    "Unknown shell {}. Must be one of Bash, Fish, Zsh, PowerShell, or Elvish.",
                    shell.code_str(),
                ),
                reason: None,
            });
        }
    };

    // Write the script to STDOUT.
    cli().gen_completions_to(BIN_NAME, shell_variant, &mut stdout());

    // If we made it this far, nothing went wrong.
    Ok(())
}

// Program entrypoint
fn entry() -> Result<(), Error> {
    // Parse command-line arguments.
    let matches = cli().get_matches();

    // Decide what to do based on the subcommand.
    match matches.subcommand_name() {
        // [tag:check_subcommand]
        Some(subcommand) if subcommand == CHECK_SUBCOMMAND => {
            // Determine the path to the schema file.
            let schema_path = Path::new(
                matches
                    .subcommand_matches(CHECK_SUBCOMMAND)
                    .unwrap() // [ref:check_subcommand]
                    .value_of(CHECK_SUBCOMMAND_PATH_OPTION)
                    // [ref:check_subcommand_shell_required]
                    .unwrap(),
            );

            // Check the schema.
            check_schema(schema_path)?;
        }

        // [tag:shell_completion_subcommand]
        Some(subcommand) if subcommand == SHELL_COMPLETION_SUBCOMMAND => {
            shell_completion(
                matches
                    .subcommand_matches(SHELL_COMPLETION_SUBCOMMAND)
                    .unwrap() // [ref:shell_completion_subcommand]
                    .value_of(SHELL_COMPLETION_SUBCOMMAND_SHELL_OPTION)
                    // [ref:shell_completion_subcommand_shell_required]
                    .unwrap(),
            )?;
        }

        // We should never end up in this branch, provided we handled all the subcommands
        // above.
        Some(_) => panic!("Subcommand not implemented."),

        // If no subcommand was provided, the help message should have been printed
        // [ref:subcommand_required_else_help].
        None => panic!("The help message should have been printed."),
    }

    // If we made it this far, nothing went wrong.
    Ok(())
}

// Let the fun begin!
fn main() {
    // Jump to the entrypoint and report any resulting errors.
    if let Err(e) = entry() {
        eprintln!("{}", e);
        exit(1);
    }
}
