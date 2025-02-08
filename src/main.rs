use clap::Parser;
use serde::Serialize;
mod remove_whitespace;

use std::{fmt::{self}, io::{self, Read}};

fn read_from_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?; // Read all of stdin to the string buffer
    Ok(buffer)
}

#[derive(
    clap::ValueEnum, Clone, Default, Debug, Serialize,
)]
#[serde(rename_all = "kebab-case")]
enum Whitespace {
    #[default]
    Remove,
    Keep,
}

// Implement Display for Whitespace (required for ToString)
impl fmt::Display for Whitespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Whitespace::Remove => write!(f, "remove"),
            Whitespace::Keep => write!(f, "keep"),
        }
    }
}


#[derive(Parser)]
#[clap(
    name = "srch",
    about = "A CLI tool to search for values in JSON from stdin or a string."
)]
struct Cli {
    #[clap(short = 'j', long = "json-string", value_name = "JSON_STRING")]
    json_string: Option<String>,

    #[clap(value_name = "SEARCH_TERM")]
    search_term: String,

    #[clap(short = 'w', long = "whitespace", value_name = "WHITESPACE", default_value_t = Default::default())]
    whitespace: Whitespace,
}

fn main() {
    let args = Cli::parse();
    let search_term = args.search_term;

    let json_input = match args.json_string {
        Some(json_str) => {
            println!("JSON input from command line argument.");
            json_str // Use the JSON string from the argument
        }
        None => {
            println!("Reading JSON input from stdin...");
            match read_from_stdin() {
                Ok(stdin_json) => stdin_json, // Use JSON from stdin
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    std::process::exit(1); // Exit with an error code
                }
            }
        }
    };

    println!("JSON Input:\n{}", match args.whitespace {
        Whitespace::Remove => { remove_whitespace::remove_whitespace(&json_input) },
        Whitespace::Keep => { json_input },
    }); // For debugging, print the input
    println!("Search Term: {}", match args.whitespace {
        Whitespace::Remove => { remove_whitespace::remove_whitespace(&search_term) },
        Whitespace::Keep => { search_term },
    });

    // TODO: Parse JSON and implement search logic
}
