use regex::Regex;
use serde_json::Value;

fn search_json_value(
    json_value: &Value,
    field_path_parts: &[&str],
    field_name: &str,
    search_regex: &Regex,
    current_path: Vec<String>,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    if single {
        // Early return if single mode and already found a match (by checking results later)
        let mut temp_results: Vec<String> = Vec::new(); // Create a temp results to check for emptiness later
        match json_value {
            Value::Object(obj) => temp_results.extend(search_object(
                obj,
                field_path_parts,
                field_name,
                search_regex,
                current_path,
                single,
                hide_value,
                field_path_separator,
            )),
            Value::Array(arr) => temp_results.extend(search_array(
                arr,
                field_path_parts,
                field_name,
                search_regex,
                current_path,
                single,
                hide_value,
                field_path_separator,
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
        // Not single mode
        match json_value {
            Value::Object(obj) => search_object(
                obj,
                field_path_parts,
                field_name,
                search_regex,
                current_path,
                single,
                hide_value,
                field_path_separator,
            ),
            Value::Array(arr) => search_array(
                arr,
                field_path_parts,
                field_name,
                search_regex,
                current_path,
                single,
                hide_value,
                field_path_separator,
            ),
            _ => Vec::new(), // For other types like String, Number, Bool, return empty results
        }
    }
}

fn search_object(
    obj: &serde_json::Map<String, Value>,
    field_path_parts: &[&str],
    field_name: &str,
    search_regex: &Regex,
    current_path: Vec<String>,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    let mut results = Vec::new();
    let mut next_path = current_path.clone();

    for (key, value) in obj {
        next_path.push(key.clone());
        let recursive_results = search_json_value(
            value,
            field_path_parts,
            field_name,
            search_regex,
            next_path.clone(),
            single,
            hide_value,
            field_path_separator,
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
        search_regex,
        &current_path,
        hide_value,
        field_path_separator,
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
    search_regex: &Regex,
    current_path: &Vec<String>,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    let mut results = Vec::new();
    let path_matches = if !field_path_parts.is_empty() {
        let mut current_path_index = 0;
        let mut field_path_index = 0;

        while current_path_index < current_path.len() && field_path_index < field_path_parts.len() {
            if current_path[current_path_index] == field_path_parts[field_path_index] {
                field_path_index += 1;
            }
            current_path_index += 1;
        }
        field_path_index == field_path_parts.len()
    } else {
        true // field_path_parts is empty, so path always matches
    };

    if path_matches {
        if let Some(value) = obj.get(field_name) {
            if search_regex.is_match(&value_to_string(value).trim_matches('"')) {
                let mut full_path = current_path.join(field_path_separator)
                    + (if current_path.is_empty() {
                        ""
                    } else {
                        field_path_separator
                    })
                    + field_name;
                if !hide_value {
                    full_path.push_str(": ");
                    full_path.push_str(&value_to_string(value).trim_matches('"'));
                }
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
    search_regex: &Regex,
    current_path: Vec<String>,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    let mut results = Vec::new();
    for (index, item) in arr.iter().enumerate() {
        let mut next_path = current_path.clone();
        next_path.push(index.to_string()); // Add array index to path
        let recursive_results = search_json_value(
            item,
            field_path_parts,
            field_name,
            search_regex,
            next_path,
            single,
            hide_value,
            field_path_separator,
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

pub fn process_json_input(
    json_input_raw: String,
    field_path_parts: &[&str],
    field_name: &str,
    search_regex: &Regex,
    single: bool,
    hide_value: bool,
    field_path_separator: &str,
) -> Vec<String> {
    match serde_json::from_str(&json_input_raw) {
        Ok(json_value) => {
            search_json_value(
                &json_value,
                field_path_parts,
                field_name,
                search_regex,
                Vec::new(), // Initial path is empty
                single,
                hide_value,
                field_path_separator,
            )
        }
        Err(e) => {
            eprintln!("Error parsing JSON input: {}", e);
            Vec::new() // Return empty results on parsing error, avoid program exit in processing files
        }
    }
}
