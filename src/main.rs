use clap::Parser;
use regex::Regex;

mod cli;
mod file;
mod parse;
mod syntax;

use cli::Cli;
use file::{handle_file_input, handle_string_or_stdin_input};
use parse::SearchContext;
use syntax::parse_search_path;

fn main() {
    let args = Cli::parse();
    let search_path_raw = args.search_path;
    let search_term = args.search_term;
    let json_files = args.json_files;
    let json_string = args.json_string;
    let path_output = args.path_output;

    match Regex::new(&search_term) {
        Ok(search_regex) => {
            let search_context = SearchContext {
                search_regex: &search_regex,
                single: args.single,
                hide_value: args.hide_value,
                field_path_separator: &args.field_path_separator,
            };
            match parse_search_path(&search_path_raw, search_context.field_path_separator) {
                Ok((field_path_parts, field_name)) => {
                    if !json_files.is_empty() {
                        handle_file_input(
                            &json_files,
                            &field_path_parts,
                            field_name,
                            &search_context,
                            path_output,
                        );
                    } else {
                        handle_string_or_stdin_input(
                            &json_string,
                            &field_path_parts,
                            field_name,
                            &search_context,
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing search path: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing search term as regex: {}", e);
            std::process::exit(1);
        }
    }
}
