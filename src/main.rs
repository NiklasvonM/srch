use clap::Parser;
use regex::Regex;

mod cli;
mod file;
mod parse;
mod syntax;

use cli::Cli;
use file::{handle_file_input, handle_string_or_stdin_input};
use syntax::parse_search_path;

fn main() {
    let args = Cli::parse();
    let search_path_raw = args.search_path;
    let search_term = args.search_term;
    let json_files = args.json_files;
    let json_string = args.json_string;
    let single = args.single;
    let path_output = args.path_output;
    let field_path_separator = args.field_path_separator;
    let hide_value = args.hide_value;

    match Regex::new(&search_term) {
        Ok(search_regex) => match parse_search_path(&search_path_raw, &field_path_separator) {
            Ok((field_path_parts, field_name)) => {
                if !json_files.is_empty() {
                    handle_file_input(
                        &json_files,
                        &field_path_parts,
                        field_name,
                        &search_regex,
                        single,
                        path_output,
                        hide_value,
                    );
                } else {
                    handle_string_or_stdin_input(
                        &json_string,
                        &field_path_parts,
                        field_name,
                        &search_regex,
                        single,
                        hide_value,
                    );
                }
            }
            Err(e) => {
                eprintln!("Error parsing search path: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error parsing search term as regex: {}", e);
            std::process::exit(1);
        }
    }
}
