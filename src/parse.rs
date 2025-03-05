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
) -> Option<Vec<String>> {
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
        _ => None, // For other types like String, Number, Bool, no further search, return None
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
) -> Option<Vec<String>> {
    let mut results: Vec<String> = Vec::new();
    let mut next_path = current_path.clone();

    for (key, value) in obj {
        next_path.push(key.clone());
        if let Some(recursive_results) = search_json_value(
            value,
            field_path_parts,
            field_name,
            search_regex,
            next_path.clone(),
            single,
            hide_value,
            field_path_separator,
        ) {
            if single {
                return Some(recursive_results); // Early return if single and found result in recursion
            }
            results.extend(recursive_results); // Otherwise extend all results.
        }
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
        return Some(check_results); // Early return if single and found result in check
    }
    if !check_results.is_empty() {
        results.extend(check_results);
    }

    if !results.is_empty() || !single && !results.is_empty() {
        // In non-single mode, return all found results at this level and below.
        // In single mode, if we found anything at this level or below, return it.
        Some(results)
    } else {
        None // No results found in this object and its children
    }
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
                let full_path = format!(
                    "{}{}{}",
                    current_path.join(field_path_separator),
                    if current_path.is_empty() { "" } else { field_path_separator },
                    field_name
                );
                let output_string = if hide_value {
                    full_path
                } else {
                    format!("{}: {}", full_path, value_to_string(value).trim_matches('"'))
                };
                results.push(output_string);
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
) -> Option<Vec<String>> {
    let mut results: Vec<String> = Vec::new();
    for (index, item) in arr.iter().enumerate() {
        let mut next_path = current_path.clone();
        next_path.push(index.to_string()); // Add array index to path
        if let Some(recursive_results) = search_json_value(
            item,
            field_path_parts,
            field_name,
            search_regex,
            next_path,
            single,
            hide_value,
            field_path_separator,
        ) {
            if single {
                return Some(recursive_results); // Early return if single and found result in recursion
            }
            results.extend(recursive_results); // Otherwise extend all results.
        }
    }

    if !results.is_empty() || !single && !results.is_empty() {
        // In non-single mode, return all found results at this level and below.
        // In single mode, if we found anything at this level or below, return it.
        Some(results)
    } else {
        None // No results found in this array and its children
    }
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
) -> Option<Vec<String>> {
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
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use serde_json::json;

    #[test]
    fn test_search_json_value_single_match_object() {
        let json_value = json!({
            "a": {
                "b": {
                    "c": "test"
                }
            }
        });
        let field_path_parts = &["a", "b"];
        let field_name = "c";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            true,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["a.b.c: test"]);
    }

    #[test]
    fn test_search_json_value_single_match_array() {
        let json_value = json!([
            {"a": "test1"},
            {"a": "test2"}
        ]);
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("test2").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            true,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["1.a: test2"]);
    }

    #[test]
    fn test_search_json_value_multiple_matches_object() {
        let json_value = json!({
            "a": {
                "b": "test",
                "c": "test"
            }
        });
        let field_path_parts = &["a"];
        let field_name = "b";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["a.b: test"]);
    }

    #[test]
    fn test_search_json_value_multiple_matches_array() {
        let json_value = json!([
            {"a": "test"},
            {"a": "test"}
        ]);
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["0.a: test", "1.a: test"]);
    }

    #[test]
    fn test_search_json_value_no_match() {
        let json_value = json!({"a": "value"});
        let field_path_parts = &[];
        let field_name = "b";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, [] as [&str; 0]);
    }

    #[test]
    fn test_search_json_value_hide_value() {
        let json_value = json!({"a": "test"});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            false,
            true,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["a"]);
    }

    #[test]
    fn test_search_json_value_field_path_match() {
        let json_value = json!({"a":{"b":{"c":"test"}}});
        let field_path_parts = &["a", "b"];
        let field_name = "c";
        let search_regex = Regex::new("test").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            &search_regex,
            Vec::new(),
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["a.b.c: test"]);
    }

    #[test]
    fn test_process_json_input_valid() {
        let json_input = r#"{"a": "test"}"#.to_string();
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("test").unwrap();
        let results = process_json_input(
            json_input,
            field_path_parts,
            field_name,
            &search_regex,
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, vec!["a: test"]);
    }

    #[test]
    fn test_process_json_input_invalid() {
        let json_input = r#"{invalid json"#.to_string();
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("test").unwrap();
        let results = process_json_input(
            json_input,
            field_path_parts,
            field_name,
            &search_regex,
            false,
            false,
            ".",
        )
        .unwrap_or_default();
        assert_eq!(results, [] as [&str; 0]);
    }
}
