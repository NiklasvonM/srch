use serde_json::Value;

// Parse search term syntax
pub fn parse_search_term(search_term: &str) -> Result<(Vec<&str>, &str, &str), String> {
    if let Some((path_part, condition_part)) = search_term.split_once(':') {
        if let Some((field_path_str, field_name)) = path_part.rsplit_once('.') {
            let expected_value = condition_part.trim();
            if !field_name.is_empty() && !expected_value.is_empty() {
                let field_path_parts: Vec<&str> = field_path_str.split('.').collect();
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

pub fn process_json_input(
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
