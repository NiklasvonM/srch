use regex::Regex;

use std::fs;
use std::io::{self, BufReader, Read};

use crate::parse::process_json_input;

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
    search_regex: &Regex,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    match fs::read_to_string(file_path) {
        Ok(file_content) => {
            let results = process_json_input(
                file_content,
                field_path_parts,
                field_name,
                search_regex,
                single,
                hide_value,
                field_path_separator,
            );
            match results {
                Some(result_vec) => result_vec,
                None => Vec::new(),
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            Vec::new() // Return empty results on file reading error, continue with other files
        }
    }
}

pub fn handle_file_input(
    json_files: &Vec<String>,
    field_path_parts: &[&str],
    field_name: &str,
    search_regex: &Regex,
    single: bool,
    path_output: bool,
    hide_value: bool,
    field_path_separator: &str,
) {
    for file_path in json_files {
        let search_results = process_file(
            file_path,
            field_path_parts,
            field_name,
            search_regex,
            single,
            hide_value,
            field_path_separator,
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

pub fn handle_string_or_stdin_input(
    json_string: &Option<String>,
    field_path_parts: &[&str],
    field_name: &str,
    search_regex: &Regex,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
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

    if let Some(search_results) = process_json_input(
        json_input_raw,
        field_path_parts,
        field_name,
        search_regex,
        single,
        hide_value,
        field_path_separator,
    ) {
        for result_path in search_results {
            println!("{}", result_path);
        }
    }
}
