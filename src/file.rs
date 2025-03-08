use std::fs;
use std::io::{self, BufReader, Read};

use crate::format::format_text_output;
use crate::parse::{process_json_input, SearchContext, SearchResult};

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
    search_context: &SearchContext,
) -> Vec<SearchResult> {
    match fs::read_to_string(file_path) {
        Ok(file_content) => {
            match process_json_input(file_content, field_path_parts, field_name, search_context) {
                Some(results) => results,
                None => Vec::new(),
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            Vec::new()
        }
    }
}

pub fn handle_file_input(
    json_files: &Vec<String>,
    field_path_parts: &[&str],
    field_name: &str,
    search_context: &SearchContext,
    path_output: bool,
    hide_value: bool,
) {
    for file_path in json_files {
        let search_results = process_file(file_path, field_path_parts, field_name, search_context);
        for result in search_results {
            let output = format_text_output(&result, hide_value, path_output, Some(file_path));
            println!("{}", output);
        }
    }
}

pub fn handle_string_or_stdin_input(
    json_string: &Option<String>,
    field_path_parts: &[&str],
    field_name: &str,
    search_context: &SearchContext,
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

    if let Some(search_results) =
        process_json_input(json_input_raw, field_path_parts, field_name, search_context)
    {
        for result in search_results {
            let output = format_text_output(&result, hide_value, false, None); // path_output is always false for string/stdin
            println!("{}", output);
        }
    }
}
