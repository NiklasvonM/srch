use clap::Parser;
use serde_json::Value;

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
) -> Vec<String> {
    let mut results = Vec::new();

    match json_value {
        Value::Object(obj) => {
            let mut next_path = current_path.clone();
            for (key, value) in obj {
                next_path.push(key.clone());
                results.extend(search_json_value(
                    value,
                    field_path_parts,
                    field_name,
                    expected_value,
                    next_path.clone(),
                ));
                next_path.pop(); // Backtrack for next key
            }

            // Check for match at the current level
            if !field_path_parts.is_empty() {
                let path_len = field_path_parts.len();
                let current_path_len = current_path.len();

                if current_path_len >= path_len {
                    // check if current path is long enough to potentially end with field_path
                    let path_suffix: Vec<String> = current_path
                        .iter()
                        .skip(current_path_len - path_len)
                        .cloned()
                        .collect(); // Take suffix of current path
                    if path_suffix
                        .iter()
                        .zip(field_path_parts.iter())
                        .all(|(a, b)| a == *b)
                    {
                        // Check if current path *ends* with field_path
                        if let Some(value) = obj.get(field_name) {
                            if value_to_string(value).trim_matches('"') == expected_value {
                                let full_path = current_path.join(".") + "." + field_name;
                                results.push(full_path);
                            }
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
        }
        Value::Array(arr) => {
            for (index, item) in arr.iter().enumerate() {
                let mut next_path = current_path.clone();
                next_path.push(index.to_string()); // Add array index to path
                results.extend(search_json_value(
                    item,
                    field_path_parts,
                    field_name,
                    expected_value,
                    next_path,
                ));
            }
        }
        _ => {}
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

fn main() {
    let args = Cli::parse();
    let search_term_raw = args.search_term;
    let json_files = args.json_files;

    match parse_search_term(&search_term_raw) {
        Ok((field_path_parts, field_name, expected_value)) => {
            if !json_files.is_empty() {
                for file_path in json_files {
                    match std::fs::read_to_string(&file_path) {
                        Ok(file_content) => match serde_json::from_str(&file_content) {
                            Ok(json_value) => {
                                let search_results = search_json_value(
                                    &json_value,
                                    &field_path_parts,
                                    field_name,
                                    &expected_value,
                                    Vec::new(), // Initial path is empty
                                );
                                for result_path in search_results {
                                    println!("{}", result_path);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error parsing JSON in file '{}': {}", file_path, e)
                            }
                        },
                        Err(e) => eprintln!("Error reading file '{}': {}", file_path, e),
                    }
                }
            } else {
                let json_input_raw = match args.json_string {
                    Some(json_str) => json_str,
                    None => match read_from_stdin() {
                        Ok(stdin_json) => stdin_json,
                        Err(e) => {
                            eprintln!("Error reading from stdin: {}", e);
                            std::process::exit(1);
                        }
                    },
                };

                match serde_json::from_str(&json_input_raw) {
                    Ok(json_value) => {
                        let search_results = search_json_value(
                            &json_value,
                            &field_path_parts,
                            field_name,
                            &expected_value,
                            Vec::new(), // Initial path is empty
                        );
                        for result_path in search_results {
                            println!("{}", result_path);
                        }
                    }
                    Err(e) => eprintln!("Error parsing JSON input: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing search term: {}", e);
            std::process::exit(1);
        }
    }
}
