mod cli;
use clap::Parser;
use cli::Cli;

mod parse;
use parse::{parse_search_term, process_json_input};

use std::fs;
use std::io::{self, BufReader, Read};

fn read_from_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    reader.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn process_file(
    file_path: &str,
    field_path_parts: &[&str],
    field_name: &str,
    expected_values: &[&str],
    single: bool,
    hide_value: bool,
) -> Vec<String> {
    match fs::read_to_string(file_path) {
        Ok(file_content) => {
            let results = process_json_input(
                file_content,
                field_path_parts,
                field_name,
                expected_values,
                single,
                hide_value,
            );
            results
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            Vec::new() // Return empty results on file reading error, continue with other files
        }
    }
}

fn handle_file_input(
    json_files: &Vec<String>,
    field_path_parts: &[&str],
    field_name: &str,
    expected_values: &[&str],
    single: bool,
    path_output: bool,
    hide_value: bool,
) {
    for file_path in json_files {
        let search_results = process_file(
            file_path,
            field_path_parts,
            field_name,
            expected_values,
            single,
            hide_value,
        );
        for result_path in search_results {
            if path_output {
                println!("{}", file_path);
            } else {
                println!("{}", result_path);
            }
            if single {
                break; // Exit inner loop after first result in single mode
            }
        }
    }
}

fn handle_string_or_stdin_input(
    json_string: &Option<String>,
    field_path_parts: &[&str],
    field_name: &str,
    expected_values: &[&str],
    single: bool,
    hide_value: bool,
) {
    let json_input_raw = match json_string {
        Some(json_str) => json_str.clone(),
        None => match read_from_stdin() {
            Ok(stdin_json) => stdin_json,
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            }
        },
    };

    let search_results = process_json_input(
        json_input_raw,
        field_path_parts,
        field_name,
        expected_values,
        single,
        hide_value,
    );
    for result_path in search_results {
        println!("{}", result_path);
    }
}

fn main() {
    let args = Cli::parse();
    let search_term_raw = args.search_term;
    let json_files = args.json_files;
    let json_string = args.json_string;
    let single = args.single;
    let path_output = args.path_output;
    let field_path_separator = args.field_path_separator;
    let value_separator = args.value_separator;
    let hide_value = args.hide_value;

    match parse_search_term(&search_term_raw, &field_path_separator, &value_separator) {
        Ok((field_path_parts, field_name, expected_values)) => {
            if !json_files.is_empty() {
                handle_file_input(
                    &json_files,
                    &field_path_parts,
                    field_name,
                    &expected_values,
                    single,
                    path_output,
                    hide_value,
                );
            } else {
                handle_string_or_stdin_input(
                    &json_string,
                    &field_path_parts,
                    field_name,
                    &expected_values,
                    single,
                    hide_value,
                );
            }
        }
        Err(e) => {
            eprintln!("Error parsing search term: {}", e);
            std::process::exit(1);
        }
    }
}
