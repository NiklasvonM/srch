pub fn parse_search_path<'a>(
    search_path: &'a str,
    field_path_separator: &'a str,
) -> Result<(Vec<&'a str>, &'a str), String> {
    if let Some((field_path_str, field_name)) = search_path.rsplit_once(field_path_separator) {
        if !field_name.is_empty() {
            let field_path_parts: Vec<&str> = field_path_str.split(field_path_separator).collect();
            return Ok((field_path_parts, field_name));
        } else {
            return Err(
                "Invalid search term format. Field name or expected value is empty.".to_string(),
            );
        }
    } else {
        // Handle case where there's no dot in path, e.g., "field:value" - fieldPath is empty
        let field_name = search_path;
        if !field_name.is_empty() {
            return Ok((vec![], field_name)); // Empty field_path_parts when no path
        } else {
            return Err(
                "Invalid search term format. Field name or expected value is empty.".to_string(),
            );
        }
    }
}

pub fn parse_numeric_search_term(search_term: &str) -> Option<(&str, &str)> {
    let ops = ["<=", ">=", "<", ">"];
    for op in ops {
        if let Some(num_str) = search_term.strip_prefix(op) {
            return Some((op, num_str));
        }
    }
    None
}

pub fn parse_numeric_range_term(search_term: &str) -> Option<((&str, &str), (&str, &str))> {
    let ops = ["<=", ">=", "<", ">"];
    for op1 in &ops {
        for op2 in &ops {
            // Example pattern: >10<20, >=5<=15, 10, <=25>=1
            if let Some(rest1) = search_term.strip_prefix(op1) {
                if let Some(num_str1_end_op2) = rest1.find(op2) {
                    let num_str1 = &rest1[..num_str1_end_op2];
                    let rest2 = &rest1[num_str1_end_op2..];
                    let op_str2 = &rest2[..op2.len()];
                    let num_str2 = &rest2[op2.len()..];

                    if !num_str1.is_empty() && !num_str2.is_empty() {
                        return Some(((op1, num_str1), (op_str2, num_str2)));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_path_valid_with_path() {
        let search_path = "a.b.c.field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_search_path_valid_without_path() {
        let search_path = "field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec![], "field")));
    }

    #[test]
    fn test_parse_search_path_valid_with_different_separator() {
        let search_path = "a/b/c/field";
        let field_path_separator = "/";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_search_path_empty_field_name() {
        let search_path = "a.b.c.";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_empty_search_path() {
        let search_path = "";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_only_separator() {
        let search_path = ".";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_multiple_separators_no_field_name() {
        let search_path = "a.b.c..";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_field_name_with_separator_char() {
        let search_path = "a.b.c.field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_numeric_search_term_valid() {
        assert_eq!(parse_numeric_search_term("<=10"), Some(("<=", "10")));
        assert_eq!(parse_numeric_search_term(">=20"), Some((">=", "20")));
        assert_eq!(parse_numeric_search_term("<5"), Some(("<", "5")));
        assert_eq!(parse_numeric_search_term(">25"), Some((">", "25")));
    }

    #[test]
    fn test_parse_numeric_search_term_invalid() {
        assert_eq!(parse_numeric_search_term("!=10"), None);
        assert_eq!(parse_numeric_search_term("~10"), None);
        assert_eq!(parse_numeric_search_term("=10"), None);
        assert_eq!(parse_numeric_search_term("10<"), None);
        assert_eq!(parse_numeric_search_term("10>"), None);
        assert_eq!(parse_numeric_search_term("10<="), None);
        assert_eq!(parse_numeric_search_term("10>="), None);
    }

    #[test]
    fn test_parse_numeric_search_term_no_operator() {
        assert_eq!(parse_numeric_search_term("10"), None);
        assert_eq!(parse_numeric_search_term("abc"), None);
        assert_eq!(parse_numeric_search_term(""), None);
    }

    #[test]
    fn test_parse_numeric_range_term_valid() {
        assert_eq!(
            parse_numeric_range_term(">10<20"),
            Some(((">", "10"), ("<", "20")))
        );
        assert_eq!(
            parse_numeric_range_term(">=5<=15"),
            Some(((">=", "5"), ("<=", "15")))
        );
        assert_eq!(
            parse_numeric_range_term("<=25>=1"),
            Some((("<=", "25"), (">=", "1")))
        );
        assert_eq!(
            parse_numeric_range_term(">=1<=25"),
            Some(((">=", "1"), ("<=", "25")))
        );
    }

    #[test]
    fn test_parse_numeric_range_term_invalid() {
        assert_eq!(parse_numeric_range_term(">10-20"), None);
        assert_eq!(parse_numeric_range_term("10"), None);
        assert_eq!(parse_numeric_range_term("><1020"), None);
        assert_eq!(parse_numeric_range_term("1020<>"), None);
        assert_eq!(parse_numeric_range_term("=10<20"), None); // invalid op
        assert_eq!(parse_numeric_range_term(">10=20"), None); // invalid op
    }

    #[test]
    fn test_parse_numeric_range_term_single_number_search() {
        assert_eq!(parse_numeric_range_term("10"), None);
        assert_eq!(parse_numeric_range_term("abc"), None);
    }

    #[test]
    fn test_parse_numeric_range_term_empty() {
        assert_eq!(parse_numeric_range_term(""), None);
    }

    #[test]
    fn test_parse_numeric_range_term_operators_only() {
        assert_eq!(parse_numeric_range_term("><"), None);
        assert_eq!(parse_numeric_range_term("<>"), None);
        assert_eq!(parse_numeric_range_term(">="), None);
        assert_eq!(parse_numeric_range_term("<="), None);
    }
}
