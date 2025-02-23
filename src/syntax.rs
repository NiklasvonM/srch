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
}
