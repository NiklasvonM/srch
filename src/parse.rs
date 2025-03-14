use regex::Regex;
use serde_json::Value;

use crate::syntax::NumericSearchTerm;

pub struct SearchContext<'a> {
    pub search_regex: &'a Regex,
    pub single_result_only: bool,
    pub field_path_separator: &'a str,
    pub numeric_search_enabled: bool,
}

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub json_path: Vec<String>,
    pub value: Value,
}

impl SearchResult {
    // Associated function for creating SearchResult
    fn create(current_path: &[String], field_name: &str, value: &Value) -> Self {
        let mut json_path = current_path.to_vec();
        json_path.push(field_name.to_string());

        SearchResult {
            json_path,
            value: value.clone(),
        }
    }
}

fn search_json_value(
    json_value: &Value,
    field_path_parts: &[&str],
    field_name: &str,
    current_path: Vec<String>,
    search_context: &SearchContext,
) -> Option<Vec<SearchResult>> {
    match json_value {
        Value::Object(obj) => search_object(
            obj,
            field_path_parts,
            field_name,
            current_path,
            search_context,
        ),
        Value::Array(arr) => search_array(
            arr,
            field_path_parts,
            field_name,
            current_path,
            search_context,
        ),
        _ => None, // No further search for primitives
    }
}

fn search_object(
    obj: &serde_json::Map<String, Value>,
    field_path_parts: &[&str],
    field_name: &str,
    current_path: Vec<String>,
    search_context: &SearchContext,
) -> Option<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = Vec::new();
    let mut next_path = current_path.clone();

    for (key, value) in obj {
        next_path.push(key.clone());
        if let Some(recursive_results) = search_json_value(
            value,
            field_path_parts,
            field_name,
            next_path.clone(),
            search_context,
        ) {
            results.extend(recursive_results);
            if search_context.single_result_only {
                return Some(results); // Early return in single result mode
            }
        }
        next_path.pop(); // Backtrack
    }

    if let Some(found_value) = check_object_match(
        obj,
        field_path_parts,
        field_name,
        &current_path,
        search_context,
    ) {
        results.push(found_value);
        if search_context.single_result_only {
            return Some(results);
        }
    }
    if !results.is_empty() {
        Some(results)
    } else {
        None
    }
}

fn check_object_match(
    obj: &serde_json::Map<String, Value>,
    field_path_parts: &[&str],
    field_name: &str,
    current_path: &[String],
    search_context: &SearchContext,
) -> Option<SearchResult> {
    if !path_matches(field_path_parts, current_path) {
        return None;
    }

    let value = obj.get(field_name)?;

    if search_context.numeric_search_enabled {
        check_numeric_match(value, field_name, current_path, search_context)
    } else {
        check_regex_match(value, field_name, current_path, search_context)
    }
}

fn path_matches(field_path_parts: &[&str], current_path: &[String]) -> bool {
    if field_path_parts.is_empty() {
        true
    } else {
        field_path_parts
            .iter()
            .zip(current_path.iter())
            .all(|(path_part, current_part)| path_part == current_part)
            && field_path_parts.len() <= current_path.len()
    }
}

fn check_numeric_match(
    value: &Value,
    field_name: &str,
    current_path: &[String],
    search_context: &SearchContext,
) -> Option<SearchResult> {
    if let Some(numeric_term) =
        NumericSearchTerm::from_search_term(search_context.search_regex.as_str())
    {
        if let Some(json_num) = value.as_f64() {
            if numeric_term.matches(json_num) {
                return Some(SearchResult::create(current_path, field_name, value));
            }
        }
    }
    None
}

fn check_regex_match(
    value: &Value,
    field_name: &str,
    current_path: &[String],
    search_context: &SearchContext,
) -> Option<SearchResult> {
    if (value.is_string() || value.is_number() || value.is_boolean())
        && search_context.search_regex.is_match(&value.to_string())
    {
        return Some(SearchResult::create(current_path, field_name, value));
    }

    None
}

fn search_array(
    arr: &[Value],
    field_path_parts: &[&str],
    field_name: &str,
    current_path: Vec<String>,
    search_context: &SearchContext,
) -> Option<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = Vec::new();
    for (index, item) in arr.iter().enumerate() {
        let mut next_path = current_path.clone();
        next_path.push(index.to_string()); // Add array index to path
        if let Some(recursive_results) = search_json_value(
            item,
            field_path_parts,
            field_name,
            next_path,
            search_context,
        ) {
            if search_context.single_result_only {
                return Some(recursive_results); // Early return in single result mode
            }
            results.extend(recursive_results);
        }
    }

    if !results.is_empty() {
        Some(results)
    } else {
        None
    }
}

pub fn process_json_input(
    json_input_raw: String,
    field_path_parts: &[&str],
    field_name: &str,
    search_context: &SearchContext,
) -> Option<Vec<SearchResult>> {
    match serde_json::from_str(&json_input_raw) {
        Ok(json_value) => search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            search_context,
        ),
        Err(e) => {
            eprintln!("JSON parsing error: {}", e);
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: true,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                value: json!("test")
            }],
        );
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: true,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["1".to_string(), "a".to_string()],
                value: json!("test2"),
            }],
        );
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string(), "b".to_string()],
                value: json!("test"),
            }],
        );
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![
                SearchResult {
                    json_path: vec!["0".to_string(), "a".to_string()],
                    value: json!("test"),
                },
                SearchResult {
                    json_path: vec!["1".to_string(), "a".to_string()],
                    value: json!("test"),
                },
            ],
        );
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]);
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                value: json!("test"),
            }],
        );
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
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!("test"),
            }],
        );
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
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]);
    }

    #[test]
    fn test_search_json_value_numeric_greater_than() {
        let json_value = json!({"a": 30});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">25").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!(30),
            }],
        );
    }

    #[test]
    fn test_search_json_value_numeric_less_equal() {
        let json_value = json!({"a": 10});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("<=10").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!(10),
            }],
        );
    }

    #[test]
    fn test_search_json_value_numeric_equal_no_match() {
        let json_value = json!({"a": 10});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("==11").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]);
    }

    #[test]
    fn test_search_json_value_numeric_invalid_operator() {
        let json_value = json!({"a": 10});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("~10").unwrap(); // ~ is not a valid operator
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]); // Should not match as operator is invalid/unsupported
    }

    #[test]
    fn test_search_json_value_numeric_range_within_range() {
        let json_value = json!({"a": 15});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">10<20").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!(15),
            }],
        );
    }

    #[test]
    fn test_search_json_value_numeric_range_outside_range_lower() {
        let json_value = json!({"a": 5});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">10<20").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]);
    }

    #[test]
    fn test_search_json_value_numeric_range_outside_range_upper() {
        let json_value = json!({"a": 25});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">10<20").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]);
    }

    #[test]
    fn test_search_json_value_numeric_range_boundary_lower_inclusive() {
        let json_value = json!({"a": 10});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">=10<20").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!(10),
            }],
        );
    }

    #[test]
    fn test_search_json_value_numeric_range_boundary_upper_exclusive() {
        let json_value = json!({"a": 20});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">=10<20").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]); // 20 is not smaller than 20
    }

    #[test]
    fn test_search_json_value_numeric_range_invalid_range_format() {
        let json_value = json!({"a": 15});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new("10<><20").unwrap(); // Invalid range format
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(results, vec![]); // Should not match due to invalid format
    }

    #[test]
    fn test_search_json_value_numeric_range_mixed_operators() {
        let json_value = json!({"a": 12});
        let field_path_parts = &[];
        let field_name = "a";
        let search_regex = Regex::new(">=10<=15").unwrap();
        let results = search_json_value(
            &json_value,
            field_path_parts,
            field_name,
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single_result_only: false,
                field_path_separator: ".",
                numeric_search_enabled: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: vec!["a".to_string()],
                value: json!(12),
            }],
        );
    }
}
