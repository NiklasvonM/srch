use regex::Regex;
use serde_json::Value;

use crate::syntax::{parse_numeric_range_term, parse_numeric_search_term};

pub struct SearchContext<'a> {
    pub search_regex: &'a Regex,
    pub single: bool,
    pub field_path_separator: &'a str,
    pub numeric_search: bool,
}

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub json_path: String,
    pub value: String,
}

impl SearchResult {
    fn create(
        current_path: &Vec<String>,
        field_name: &str,
        value: &Value,
        search_context: &SearchContext,
    ) -> SearchResult {
        let full_path = format!(
            "{}{}{}",
            current_path.join(search_context.field_path_separator),
            if current_path.is_empty() {
                ""
            } else {
                search_context.field_path_separator
            },
            field_name
        );
        SearchResult {
            json_path: full_path,
            value: value_to_string(value).trim_matches('"').to_string(),
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
        _ => None, // For other types like String, Number, Bool, no further search, return None
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
            if search_context.single {
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
        &current_path,
        search_context,
    );
    if search_context.single && !check_results.is_empty() {
        return Some(check_results); // Early return if single and found result in check
    }
    if !check_results.is_empty() {
        results.extend(check_results);
    }

    if !results.is_empty() {
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
    current_path: &Vec<String>,
    search_context: &SearchContext,
) -> Vec<SearchResult> {
    let mut results: Vec<SearchResult> = Vec::new();
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

    if !path_matches {
        return results;
    }

    let Some(value) = obj.get(field_name) else {
        return results; // Field name not found, return empty results
    };

    if search_context.numeric_search {
        if let Some(((op1, num_str1), (op2, num_str2))) =
            parse_numeric_range_term(search_context.search_regex.as_str())
        {
            if let (Ok(target_num1), Ok(target_num2), Some(json_num)) = (
                num_str1.parse::<f64>(),
                num_str2.parse::<f64>(),
                value.as_f64(),
            ) {
                if compare_number_range(json_num, target_num1, op1, target_num2, op2) {
                    results.push(SearchResult::create(
                        current_path,
                        field_name,
                        value,
                        search_context,
                    ));
                }
            } else if let Some((op, num_str)) =
                parse_numeric_search_term(search_context.search_regex.as_str())
            {
                // Fallback to single numeric comparison if range parsing fails
                if let (Ok(target_num), Some(json_num)) = (num_str.parse::<f64>(), value.as_f64()) {
                    if compare_numbers(json_num, target_num, op) {
                        results.push(SearchResult::create(
                            current_path,
                            field_name,
                            value,
                            search_context,
                        ));
                    }
                }
            }
        } else if let Some((op, num_str)) =
            parse_numeric_search_term(search_context.search_regex.as_str())
        {
            // Fallback to single numeric comparison if range parsing fails
            if let (Ok(target_num), Some(json_num)) = (num_str.parse::<f64>(), value.as_f64()) {
                if compare_numbers(json_num, target_num, op) {
                    results.push(SearchResult::create(
                        current_path,
                        field_name,
                        value,
                        search_context,
                    ));
                }
            }
        }
    } else {
        // Fallback to regex search if not numeric search
        if search_context
            .search_regex
            .is_match(&value_to_string(value).trim_matches('"'))
        {
            results.push(SearchResult::create(
                current_path,
                field_name,
                value,
                search_context,
            ));
        }
    }

    results
}

fn compare_numbers(json_num: f64, target_num: f64, op: &str) -> bool {
    match op {
        ">" => json_num > target_num,
        "<" => json_num < target_num,
        ">=" => json_num >= target_num,
        "<=" => json_num <= target_num,
        "==" => json_num == target_num,
        _ => false,
    }
}

fn compare_number_range(
    json_num: f64,
    target_num1: f64,
    op1: &str,
    target_num2: f64,
    op2: &str,
) -> bool {
    let condition1 = compare_numbers(json_num, target_num1, op1);
    let condition2 = compare_numbers(json_num, target_num2, op2);

    condition1 && condition2
}

fn search_array(
    arr: &Vec<Value>,
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
            if search_context.single {
                return Some(recursive_results); // Early return if single and found result in recursion
            }
            results.extend(recursive_results); // Otherwise extend all results.
        }
    }

    if !results.is_empty() {
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
    search_context: &SearchContext,
) -> Option<Vec<SearchResult>> {
    match serde_json::from_str(&json_input_raw) {
        Ok(json_value) => {
            search_json_value(
                &json_value,
                field_path_parts,
                field_name,
                Vec::new(), // Initial path is empty
                search_context,
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
            Vec::new(),
            &SearchContext {
                search_regex: &search_regex,
                single: true,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a.b.c".to_string(),
                value: "test".to_string(),
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
                single: true,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "1.a".to_string(),
                value: "test2".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a.b".to_string(),
                value: "test".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![
                SearchResult {
                    json_path: "0.a".to_string(),
                    value: "test".to_string(),
                },
                SearchResult {
                    json_path: "1.a".to_string(),
                    value: "test".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a.b.c".to_string(),
                value: "test".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "test".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: false,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "30".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "10".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "15".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "10".to_string(),
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
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
                single: false,
                field_path_separator: ".",
                numeric_search: true,
            },
        )
        .unwrap_or_default();
        assert_eq!(
            results,
            vec![SearchResult {
                json_path: "a".to_string(),
                value: "12".to_string(),
            }],
        );
    }
}
