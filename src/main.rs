use clap::Parser;
use serde_json::Value;

use std::fs;
use std::io::{self, BufReader, Read};

fn read_from_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    reader.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[derive(Parser)]
#[clap(
    name = "srch",
    about = "A CLI tool to search for values in JSON from stdin, string, or files.\n\
            Example usage: `srch \"fieldPath.fieldName: true\" example_files/*.json | wc`"
)]
struct Cli {
    #[clap(
        short = 'j',
        long = "json-string",
        value_name = "JSON_STRING",
        help = "Provide JSON input as a string directly in the command line."
    )]
    json_string: Option<String>,

    #[clap(
        value_name = "SEARCH_TERM",
        help = "Search term in the format 'fieldPath.fieldName:expectedValue'.\n\
                                         - fieldPath: Path to the field, separated by dots (e.g., 'topLevel.nestedField' or just 'field').\n\
                                         - fieldName: Name of the field to search for at the end of the path.\n\
                                         - expectedValue: Value to compare against. The value is compared as a string.\n\
                                         Examples: 'fieldOne.isPresent:true', 'topLevel.nested.value:123'"
    )]
    search_term: String,

    #[clap(value_name = "JSON_FILES", num_args = 0.., help = "Paths to JSON files to search within. If provided, srch will search these files instead of stdin or --json-string.")]
    json_files: Vec<String>,

    #[clap(
        short = 's',
        long = "single",
        help = "Return only the first match per file."
    )]
    single: bool,

    #[clap(
        short = 'p',
        long = "path",
        help = "Output the file path instead of the result path (only for file input)."
    )]
    path_output: bool,
}

// Parse search term syntax
fn parse_search_term(search_term: &str) -> Result<(Vec<&str>, &str, &str), String> {
    if let Some((path_part, condition_part)) = search_term.split_once(':') {
        if let Some((field_path_str, field_name)) = path_part.rsplit_once('.') {
            let field_path_parts: Vec<&str> = field_path_str.split('.').collect();
            let expected_value = condition_part.trim();
            if !field_name.is_empty() && !expected_value.is_empty() {
                return Ok((field_path_parts, field_name, expected_value));
            } else {
                return Err(
                    "Invalid search term format. Field name or expected value is empty."
                        .to_string(),
                );
            }
        } else {
            // Handle case where there's no dot in path, e.g., "field:value" - fieldPath is empty
            let field_name = path_part;
            let expected_value = condition_part.trim();
            if !field_name.is_empty() && !expected_value.is_empty() {
                return Ok((vec![], field_name, expected_value)); // Empty field_path_parts when no path
            } else {
                return Err(
                    "Invalid search term format. Field name or expected value is empty."
                        .to_string(),
                );
            }
        }
    } else {
        return Err(
            "Invalid search term format. Missing colon separator for value condition.".to_string(),
        );
    }
}

fn search_json_value(
    json_value: &Value,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    current_path: Vec<String>,
    single: bool,
) -> Vec<String> {
    if single {
        // Early return if single mode and already found a match (by checking results later)
        let mut temp_results: Vec<String> = Vec::new(); // Create a temp results to check for emptiness later
        match json_value {
            Value::Object(obj) => temp_results.extend(search_object(
                obj,
                field_path_parts,
                field_name,
                expected_value,
                current_path,
                single,
            )),
            Value::Array(arr) => temp_results.extend(search_array(
                arr,
                field_path_parts,
                field_name,
                expected_value,
                current_path,
                single,
            )),
            _ => {} // For other types like String, Number, Bool, do nothing
        }
        if !temp_results.is_empty() && single {
            // Check temp_results after the recursive call.
            return temp_results; // If temp_results is not empty, return it immediately.
        } else {
            return Vec::new(); // Otherwise return empty results to continue search (or effectively stop if no match found deeper).
        }
    } else {
        // Not single mode, process as before
        match json_value {
            Value::Object(obj) => search_object(
                obj,
                field_path_parts,
                field_name,
                expected_value,
                current_path,
                single,
            ),
            Value::Array(arr) => search_array(
                arr,
                field_path_parts,
                field_name,
                expected_value,
                current_path,
                single,
            ),
            _ => Vec::new(), // For other types like String, Number, Bool, return empty results
        }
    }
}

fn search_object(
    obj: &serde_json::Map<String, Value>,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    current_path: Vec<String>,
    single: bool,
) -> Vec<String> {
    let mut results = Vec::new();
    let mut next_path = current_path.clone();

    for (key, value) in obj {
        next_path.push(key.clone());
        let recursive_results = search_json_value(
            value,
            field_path_parts,
            field_name,
            expected_value,
            next_path.clone(),
            single,
        );
        if single && !recursive_results.is_empty() {
            // Early return if single and found result in recursion
            return recursive_results; // Return the result immediately from recursion
        }
        results.extend(recursive_results); // Otherwise extend all results.
        next_path.pop(); // Backtrack for next key
    }

    let check_results = check_object_match(
        obj,
        field_path_parts,
        field_name,
        expected_value,
        &current_path,
    );
    if single && !check_results.is_empty() {
        // Early return if single and found result in check
        return check_results; // Return result immediately from check
    }
    results.extend(check_results); // Otherwise extend all results.

    results
}

fn check_object_match(
    obj: &serde_json::Map<String, Value>,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    current_path: &Vec<String>,
) -> Vec<String> {
    let mut results = Vec::new();
    if !field_path_parts.is_empty() {
        let mut current_path_index = 0;
        let mut field_path_index = 0;

        while current_path_index < current_path.len() && field_path_index < field_path_parts.len() {
            if current_path[current_path_index] == field_path_parts[field_path_index] {
                field_path_index += 1; // Move to next field path part if match is found
            }
            current_path_index += 1; // Always move to next current path part
        }

        // Check if all parts of field_path_parts have been matched in the current_path
        if field_path_index == field_path_parts.len() {
            if let Some(value) = obj.get(field_name) {
                if value_to_string(value).trim_matches('"') == expected_value {
                    let full_path = current_path.join(".") + "." + field_name;
                    results.push(full_path);
                }
            }
        }
    } else {
        // field_path_parts is empty, search anywhere logic
        if let Some(value) = obj.get(field_name) {
            if value_to_string(value).trim_matches('"') == expected_value {
                let full_path = current_path.join(".") + "." + field_name;
                results.push(full_path);
            }
        }
    }
    results
}

fn search_array(
    arr: &Vec<Value>,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    current_path: Vec<String>,
    single: bool,
) -> Vec<String> {
    let mut results = Vec::new();
    for (index, item) in arr.iter().enumerate() {
        let mut next_path = current_path.clone();
        next_path.push(index.to_string()); // Add array index to path
        let recursive_results = search_json_value(
            item,
            field_path_parts,
            field_name,
            expected_value,
            next_path,
            single,
        );
        if single && !recursive_results.is_empty() {
            // Early return if single and found result in recursion
            return recursive_results; // Return result immediately from recursion
        }
        results.extend(recursive_results); // Otherwise extend all results.
    }
    results
}

// Helper function to convert serde_json::Value to String for comparison
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => value.to_string(),
    }
}

fn process_json_input(
    json_input_raw: String,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    single: bool,
) -> Vec<String> {
    match serde_json::from_str(&json_input_raw) {
        Ok(json_value) => {
            search_json_value(
                &json_value,
                field_path_parts,
                field_name,
                expected_value,
                Vec::new(), // Initial path is empty
                single,
            )
        }
        Err(e) => {
            eprintln!("Error parsing JSON input: {}", e);
            Vec::new() // Return empty results on parsing error, avoid program exit in processing files
        }
    }
}

fn process_file(
    file_path: &str,
    field_path_parts: &[&str],
    field_name: &str,
    expected_value: &str,
    single: bool,
) -> Vec<String> {
    match fs::read_to_string(file_path) {
        Ok(file_content) => {
            let results = process_json_input(
                file_content,
                field_path_parts,
                field_name,
                expected_value,
                single,
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
    expected_value: &str,
    single: bool,
    path_output: bool,
) {
    for file_path in json_files {
        let search_results = process_file(
            file_path,
            field_path_parts,
            field_name,
            expected_value,
            single,
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
    expected_value: &str,
    single: bool,
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
        expected_value,
        single,
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

    match parse_search_term(&search_term_raw) {
        Ok((field_path_parts, field_name, expected_value)) => {
            if !json_files.is_empty() {
                handle_file_input(
                    &json_files,
                    &field_path_parts,
                    field_name,
                    &expected_value,
                    single,
                    path_output,
                );
            } else {
                handle_string_or_stdin_input(
                    &json_string,
                    &field_path_parts,
                    field_name,
                    &expected_value,
                    single,
                );
            }
        }
        Err(e) => {
            eprintln!("Error parsing search term: {}", e);
            std::process::exit(1);
        }
    }
}
