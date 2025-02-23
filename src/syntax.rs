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